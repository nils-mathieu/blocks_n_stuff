use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

use super::Task;

/// The state shared between the task pool and worker threads.
struct Shared<T: Task> {
    /// The tasks that are ready to be executed.
    tasks: Mutex<Vec<T>>,
    /// A condvar used to wake up workers when a new task is submitted to `tasks`.
    condvar: Condvar,
    /// Whether the task pool has been requested to stop.
    stop_requested: AtomicBool,
    /// The output of the task pool.
    ///
    /// When a worker thead finishes executing a task, it will push the result on top of
    /// this vector.
    outputs: Mutex<Vec<T::Output>>,
}

impl<T: Task> Shared<T> {
    /// Creates a new [`Shared`] instance.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
            condvar: Condvar::new(),
            stop_requested: AtomicBool::new(false),
            outputs: Mutex::new(Vec::new()),
        }
    }

    /// Returns whether the task pool has been requested to stop.
    #[inline]
    pub fn should_stop(&self) -> bool {
        self.stop_requested.load(Relaxed)
    }

    /// Pushes an output value to the output vector.
    #[inline]
    pub fn push_output(&self, output: T::Output) {
        self.outputs.lock().push(output);
    }

    /// Fetches at most `count` tasks from the pool.
    ///
    /// # Returns
    ///
    /// This function returns whether new tasks where actually fetched. It will return
    /// `true` if at least one task was fetched, and `false` if the task pool was requested
    /// to stop before any tasks could be fetched.
    pub fn fetch_tasks(&self, output: &mut Vec<T>, count: usize) -> bool {
        let mut lock = self.tasks.lock();

        while !self.should_stop() {
            if lock.is_empty() {
                self.condvar.wait(&mut lock);
            } else {
                let tasks = &mut *lock;
                let to_take = count.min(tasks.len());

                output.reserve_exact(count);

                // SAFETY:
                //  We're performing this move manually because `Vec<T>` doesn't really
                //  have an API to do specifically this and I don't want to require
                //  `T: Copy`.
                //
                // This is effectively moving the `to_take` last elements of `tasks` to move
                // them at the end of `output`.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        tasks.as_ptr().add(tasks.len() - to_take),
                        output.as_mut_ptr().add(output.len()),
                        to_take,
                    );

                    tasks.set_len(tasks.len() - to_take);
                    output.set_len(output.len() + to_take);
                }

                return true;
            }
        }

        false
    }
}

/// A flavor of the task pool that uses background threads to execute tasks.
pub struct YesThreads<T: Task> {
    /// The state shared between the task pool and worker threads.
    shared: Arc<Shared<T>>,
    /// The number of threads that have been spawned.
    count: usize,
}

impl<T: Task> YesThreads<T> {
    /// Creates a new [`Shared`] instance.
    pub fn new(to_spawn: usize) -> Self
    where
        T: Send + 'static,
        T::Output: Send,
    {
        let shared = Arc::new(Shared::new());

        for _ in 0..to_spawn {
            let shared = shared.clone();
            std::thread::spawn(move || worker_thread(shared));
        }

        Self {
            shared,
            count: to_spawn,
        }
    }

    /// Request the worker threads to stop.
    pub fn stop(&self) {
        self.shared.stop_requested.store(true, Relaxed);
        self.shared.condvar.notify_all();
    }

    /// Returns the number of tasks that are waiting to be executed.
    pub fn pending_tasks(&self) -> usize {
        self.shared.tasks.lock().len()
    }

    /// Submits the provided tasks to the task pool.
    pub fn submit_batch(&mut self, tasks: &mut Vec<T>) {
        let count = tasks.len();

        self.shared.tasks.lock().append(tasks);

        if count > self.count {
            self.shared.condvar.notify_all();
        } else {
            for _ in 0..count {
                self.shared.condvar.notify_one();
            }
        }
    }

    /// Returns an iterator over the outputs of the task pool.
    pub fn fetch_outputs(&mut self) -> impl '_ + Iterator<Item = T::Output> {
        struct Iter<'a, T>(parking_lot::MutexGuard<'a, Vec<T>>);

        impl<T> Iterator for Iter<'_, T> {
            type Item = T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.pop()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.0.len(), Some(self.0.len()))
            }
        }

        impl<T> Drop for Iter<'_, T> {
            fn drop(&mut self) {
                self.for_each(drop);
            }
        }

        Iter(self.shared.outputs.lock())
    }

    /// Retains only the tasks for which the provided function returns `true`.
    #[inline]
    pub fn retain_tasks(&mut self, mut f: impl FnMut(&T) -> bool) {
        self.shared.tasks.lock().retain(f);
    }
}

/// Runs a worker thread until it's requested to stop.
fn worker_thread<T: Task>(shared: Arc<Shared<T>>) {
    const MAX_TASK_REQUESTS: usize = 8;

    let mut private_task_list = Vec::new();

    while shared.fetch_tasks(&mut private_task_list, MAX_TASK_REQUESTS) {
        while let Some(task) = private_task_list.pop() {
            let output = task.execute();
            shared.push_output(output);

            if shared.should_stop() {
                return;
            }
        }
    }
}
