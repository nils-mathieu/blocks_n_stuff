use parking_lot::{Condvar, Mutex};
use std::collections::BinaryHeap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use crate::{Priority, Worker};

/// A task with a payload.
struct Task<T> {
    priority: Priority,
    payload: T,
}

impl<T> PartialEq for Task<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T> Eq for Task<T> {}

impl<T> PartialOrd for Task<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl<T> Ord for Task<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// A pool of tasks that can be executed by multiple threads.
pub struct TaskPool<I, O> {
    /// The list of tasks that have been pushed to the list, but not have been taken by a worker
    /// just yet.
    tasks: Mutex<BinaryHeap<Task<I>>>,
    /// The list of output values that have been produced by the workers.
    results: Mutex<Vec<O>>,
    /// A condition variable that's notified whenever a new task is pushed to the list.
    ///
    /// Also when the thread pool is dropped, this condition variable is notified to wake up all
    /// worker threads.
    condvar: Condvar,

    /// Whether the worker threads should stop.
    should_stop: AtomicBool,
}

impl<I, O> Default for TaskPool<I, O> {
    #[inline]
    fn default() -> Self {
        Self {
            tasks: Mutex::new(BinaryHeap::new()),
            results: Mutex::new(Vec::new()),
            condvar: Condvar::new(),
            should_stop: AtomicBool::new(false),
        }
    }
}

impl<I, O> TaskPool<I, O> {
    /// Determines whether a worker thread should stop.
    ///
    /// If this function returns `true`, the `workers_to_remove` counter is decremented by `1`.
    #[inline]
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Relaxed)
    }

    /// Returns the total number of tasks currently in the pool.
    #[inline]
    pub fn task_count(&self) -> usize {
        self.tasks.lock().len()
    }

    /// Fetches a task to execute.
    ///
    /// If no task is available, the function blocks until a new task is pushed to the list.
    ///
    /// If the thread must stop, `None` is returned.
    pub fn fetch_task(&self) -> Option<I> {
        if self.should_stop() {
            return None;
        }

        let mut lock = self.tasks.lock();
        loop {
            match lock.pop() {
                Some(task) => return Some(task.payload),
                None => self.condvar.wait(&mut lock),
            }

            if self.should_stop() {
                return None;
            }
        }
    }

    /// Submits a new task to be executed.
    pub fn submit(&self, payload: I, priority: Priority) {
        let mut lock = self.tasks.lock();
        lock.push(Task { payload, priority });
        self.condvar.notify_one();
    }

    /// Submits a batch of tasks to be executed.
    pub fn submit_batch(&self, iter: impl IntoIterator<Item = (I, Priority)>) {
        self.tasks.lock().extend(
            iter.into_iter()
                .map(|(payload, priority)| Task { payload, priority }),
        );
        self.condvar.notify_all();
    }

    /// Adds a result to the list of results.
    pub fn push_result(&self, output: O) {
        self.results.lock().push(output);
    }

    /// Returns an iterator over the results that were received by the [`TaskPool`].
    pub fn fetch_results(&self) -> Results<'_, O> {
        Results(self.results.lock())
    }
}

/// An iterator over the results that were received by a [`TaskPool`].
pub struct Results<'a, T>(parking_lot::MutexGuard<'a, Vec<T>>);

impl<'a, T> Iterator for Results<'a, T> {
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

impl<'a, T> ExactSizeIterator for Results<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<I, O> Drop for TaskPool<I, O> {
    fn drop(&mut self) {
        self.should_stop.store(true, Relaxed);
        self.condvar.notify_all();
    }
}

/// Spawns a new worker thread.
pub fn spawn_worker<W>(pool: Arc<TaskPool<W::Input, W::Output>>, mut worker: W)
where
    W: 'static + Send + Worker,
    W::Output: Send,
    W::Input: Send,
{
    std::thread::spawn(move || {
        while let Some(task) = pool.fetch_task() {
            let output = worker.run(task);
            pool.push_result(output);
        }
    });
}
