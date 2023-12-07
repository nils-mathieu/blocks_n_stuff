use smallvec::SmallVec;

use super::Task;

/// An implementation of the task pool that does not use threads at all.
pub struct NoThreads<T> {
    /// The list of pending tasks.
    pending: Vec<T>,
}

impl<T> NoThreads<T> {
    /// Creates a new [`NoThreads`] instance.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Submits the provided tasks to the task pool.
    #[inline]
    pub fn submit_batch(&mut self, tasks: &mut Vec<T>) {
        self.pending.append(tasks);
    }

    /// Returns the number of pending tasks.
    #[inline]
    pub fn pending_tasks(&self) -> usize {
        self.pending.len()
    }

    /// Returns an iterator over the outputs of the tasks that have completed.
    pub fn fetch_outputs(&mut self) -> impl Iterator<Item = T::Output> + '_
    where
        T: Task,
    {
        /// At most 5 tasks can be executed per request.
        const BUDGET: usize = 1;

        let mut result = SmallVec::<[T::Output; BUDGET]>::new();

        // Execute a bunch of tasks on the current thread.
        for _ in 0..BUDGET.min(self.pending.len()) {
            let output = self.pending.pop().unwrap().execute();
            result.push(output);
        }

        result.into_iter()
    }

    /// Retains only the tasks that satisfy the provided predicate.
    #[inline]
    pub fn retain_tasks(&mut self, mut f: impl FnMut(&T) -> bool) {
        self.pending.retain(f)
    }
}
