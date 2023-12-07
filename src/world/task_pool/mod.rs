mod no_threads;
mod yes_threads;

/// A task that can be executed on a thread pool.
pub trait Task {
    /// The result type of the task.
    type Output;

    /// Executes the task and returns its output.
    fn execute(self) -> Self::Output;
}

/// The flavor of the thread pool.
enum Flavor<T: Task> {
    /// No threads are used.
    ///
    /// Instead, a budget of work is done every tick.
    NoThreads,
    /// Threads are used to execute tasks in the background.
    Threads(yes_threads::YesThreads<T>),
}

/// Manages a pool of worker threads ready to execute a bunch of tasks.
pub struct TaskPool<T: Task>(Flavor<T>);

impl<T: Task> TaskPool<T> {
    /// Creates a new [`TaskPool`] instance.
    pub fn new() -> Self
    where
        T: Task + Send + 'static,
        T::Output: Send,
    {
        let flavor = match num_threads() {
            0 => Flavor::NoThreads,
            num => Flavor::Threads(yes_threads::YesThreads::new(num)),
        };

        Self(flavor)
    }

    /// Returns the number of tasks that are waiting to be executed.
    pub fn pending_tasks(&self) -> usize {
        match &self.0 {
            Flavor::NoThreads => unimplemented!(),
            Flavor::Threads(f) => f.pending_tasks(),
        }
    }

    /// Submits the provided tasks to the task pool.
    pub fn submit_batch(&mut self, tasks: &mut Vec<T>) {
        match &mut self.0 {
            Flavor::NoThreads => unimplemented!(),
            Flavor::Threads(f) => f.submit_tasks(tasks),
        }
    }

    /// Returns an iterator over the outputs of the tasks that have completed.
    ///
    /// # Remarks
    ///
    /// The returned iterator might hold a lock to an internal queue, so it's
    /// generally a good idea to quickly consume and drop the iterator to avoid
    /// blocking the task pool.
    pub fn fetch_outputs(&mut self) -> impl Iterator<Item = T::Output> + '_ {
        match &mut self.0 {
            Flavor::NoThreads => unimplemented!(),
            Flavor::Threads(f) => f.fetch_outputs(),
        }
    }
}

impl<T: Task> Drop for TaskPool<T> {
    fn drop(&mut self) {
        match &self.0 {
            Flavor::NoThreads => (),
            Flavor::Threads(shared) => shared.stop(),
        }
    }
}

/// Returns the number of threads that should be used for the task pool.
///
/// Returns 0 or 1 if the task pool should not use threads at all. This is the case
/// on single-core machines, or on web.
fn num_threads() -> usize {
    match std::thread::available_parallelism() {
        Ok(num) => {
            let num = num.get();

            if num <= 2 {
                // If the has no or few parallelism available, don't use threads
                // at all.
                0
            } else {
                // Avoid using ALL available threads because that would probably
                // just eat up all the CPU resources and make the game laggy.
                // If the user cannot
                (num / 2).max(2)
            }
        }
        // We don't know how much parallelism is available, let's be conservative
        // and assume there isn't any.
        Err(_) => 0,
    }
}
