use parking_lot::{Condvar, Mutex};
use std::collections::BinaryHeap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use crate::{Priority, Worker};

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

/// The state that's shared between all workers.
struct Shared<I, O> {
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

impl<I, O> Shared<I, O> {
    /// Determines whether a worker thread should stop.
    ///
    /// If this function returns `true`, the `workers_to_remove` counter is decremented by `1`.
    #[inline]
    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Relaxed)
    }

    /// Fetches a task to execute.
    ///
    /// If no task is available, the function blocks until a new task is pushed to the list.
    ///
    /// If the thread must stop, `None` is returned.
    fn fetch_task(&self) -> Option<I> {
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

    /// Adds a result to the list of results.
    fn push_result(&self, output: O) {
        self.results.lock().push(output);
    }
}

/// A pool of tasks that can be executed by multiple threads.
pub struct TaskPool<W: Worker> {
    /// The shared state.
    shared: Arc<Shared<W::Input, W::Output>>,
}

impl<W: Worker> Default for TaskPool<W> {
    #[inline]
    fn default() -> Self {
        Self {
            shared: Arc::new(Shared {
                tasks: Mutex::new(BinaryHeap::new()),
                results: Mutex::new(Vec::new()),
                condvar: Condvar::new(),
                should_stop: AtomicBool::new(false),
            }),
        }
    }
}

/// Public methods that will be called by the main implementation.
impl<W: Worker> TaskPool<W> {
    #[inline]
    pub fn task_count(&self) -> usize {
        self.shared.tasks.lock().len()
    }

    pub fn submit(&self, payload: W::Input, priority: Priority) {
        let mut lock = self.shared.tasks.lock();
        lock.push(Task { payload, priority });
        self.shared.condvar.notify_one();
    }

    pub fn submit_batch(&self, iter: impl IntoIterator<Item = (W::Input, Priority)>) {
        self.shared.tasks.lock().extend(
            iter.into_iter()
                .map(|(payload, priority)| Task { payload, priority }),
        );
        self.shared.condvar.notify_all();
    }

    pub fn fetch_results(&self) -> impl Iterator<Item = W::Output> + '_ {
        Results(self.shared.results.lock())
    }

    pub fn spawn(&self, mut worker: W)
    where
        W: Send + 'static,
        W::Input: Send,
        W::Output: Send,
    {
        let shared = self.shared.clone();
        std::thread::spawn(move || {
            while let Some(input) = shared.fetch_task() {
                let output = worker.run(input);
                shared.push_result(output);
            }
        });
    }
}

impl<W: Worker> Drop for TaskPool<W> {
    fn drop(&mut self) {
        self.shared.should_stop.store(true, Relaxed);
        self.shared.condvar.notify_all();
    }
}

/// An iterator over the results that were received by a [`TaskPool`].
struct Results<'a, T>(parking_lot::MutexGuard<'a, Vec<T>>);

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
