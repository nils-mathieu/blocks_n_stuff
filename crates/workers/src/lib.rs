//! A worker abstraction to create thread pools.

use std::sync::Arc;

#[cfg_attr(not(target_arch = "wasm32"), path = "parking_lot.rs")]
#[cfg_attr(target_arch = "wasm32", path = "web_sys.rs")]
mod imp;

/// The type used to represent the priority of a task.
pub type Priority = i32;

/// Describes the state of a worker.
///
/// This trait can be implemented by the user to provide custom worker implementations.
pub trait Worker {
    /// The input type of the worker.
    type Input;
    /// The output type of the worker.
    type Output;

    /// Runs the worker with the provided input.
    fn run(&mut self, input: Self::Input) -> Self::Output;
}

/// The inner task pool.
#[derive(Clone)]
pub struct TaskPool<I, O> {
    /// The inner task pool.
    inner: Arc<imp::TaskPool<I, O>>,
}

impl<I, O> TaskPool<I, O> {
    /// Submits a new task to the task pool.
    #[inline]
    pub fn submit(&self, input: I, priority: Priority) {
        self.inner.submit(input, priority);
    }

    /// Submits a batch of tasks to the task pool.
    #[inline]
    pub fn submit_batch(&self, iter: impl IntoIterator<Item = (I, Priority)>) {
        self.inner.submit_batch(iter);
    }

    /// Returns the tasks that have been received from workers.
    ///
    /// # Remarks
    ///
    /// The returned iterator may hold a lock to an internal queue, so you better don't hold on to
    /// it for too long.
    #[inline]
    pub fn fetch_results(&self) -> impl Iterator<Item = O> + '_ {
        self.inner.fetch_results()
    }

    /// Returns the number of tasks that are currently in the queue.
    #[inline]
    pub fn task_count(&self) -> usize {
        self.inner.task_count()
    }
}

/// Starts a collection of worker threads and returns a handle to the task pool.
pub fn start<W, I>(workers: I) -> TaskPool<W::Input, W::Output>
where
    I: IntoIterator<Item = W>,
    W: 'static + Send + Worker,
    W::Input: Send,
    W::Output: Send,
{
    let pool = Arc::new(imp::TaskPool::default());

    for worker in workers {
        let pool = pool.clone();
        imp::spawn_worker(pool, worker);
    }

    TaskPool { inner: pool }
}
