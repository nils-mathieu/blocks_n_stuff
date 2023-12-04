//! A worker abstraction to create thread pools.

// #[cfg_attr(not(target_arch = "wasm32"), path = "parking_lot.rs")]
// #[cfg_attr(target_arch = "wasm32", path = "no_workers.rs")]
#[path = "no_workers.rs"]
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

/// A trait that requires `Send` on non-WASM targets.
pub trait WasmNonSend {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> WasmNonSend for T {}

#[cfg(target_arch = "wasm32")]
impl<T> WasmNonSend for T {}

/// A task pool that allows submitting tasks to be executed by a worker.
pub struct TaskPool<W: Worker> {
    inner: imp::TaskPool<W>,
}

impl<W: Worker> Default for TaskPool<W> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: imp::TaskPool::default(),
        }
    }
}

impl<W: Worker> TaskPool<W> {
    /// Returns the number of tasks that are currently queued.
    ///
    /// Note that this does *not* include tasks that are currently running.
    pub fn task_count(&self) -> usize {
        self.inner.task_count()
    }

    /// Submits a new task to be executed by a worker with the given priority.
    ///
    /// Tasks with a higher priority will be executed first.
    pub fn submit(&self, payload: W::Input, priority: Priority) {
        self.inner.submit(payload, priority)
    }

    /// Submits multiple tasks to be executed by a worker with the given priority.
    ///
    /// Tasks with a higher priority will be executed first.
    pub fn submit_batch(&self, iter: impl IntoIterator<Item = (W::Input, Priority)>) {
        self.inner.submit_batch(iter)
    }

    /// Returns an iterator over the results of the tasks that have been executed, including
    /// their outputs.
    ///
    /// Note that the returned iterator might hold a lock to an internal queue, so it is
    /// recommended to not hold on to it for too long.
    pub fn fetch_results(&self) -> impl Iterator<Item = W::Output> + '_ {
        self.inner.fetch_results()
    }

    /// Spawns a worker that will execute tasks from the queue.
    pub fn spawn(&self, worker: W)
    where
        W: WasmNonSend + 'static,
        W::Input: WasmNonSend,
        W::Output: WasmNonSend,
    {
        self.inner.spawn(worker)
    }
}
