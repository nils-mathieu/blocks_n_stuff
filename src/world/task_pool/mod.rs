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
    NoThreads(no_threads::NoThreads<T>),
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
            0 => Flavor::NoThreads(no_threads::NoThreads::new()),
            num => Flavor::Threads(yes_threads::YesThreads::new(num)),
        };

        Self(flavor)
    }

    /// Returns the number of tasks that are waiting to be executed.
    pub fn pending_tasks(&self) -> usize {
        match &self.0 {
            Flavor::NoThreads(f) => f.pending_tasks(),
            Flavor::Threads(f) => f.pending_tasks(),
        }
    }

    /// Submits the provided tasks to the task pool.
    pub fn submit_batch(&mut self, tasks: &mut Vec<T>) {
        match &mut self.0 {
            Flavor::NoThreads(f) => f.submit_batch(tasks),
            Flavor::Threads(f) => f.submit_batch(tasks),
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
        /// An iterator that can iterator over either an A or a B.
        enum Either<L, R> {
            Left(L),
            Right(R),
        }

        impl<L, R> Iterator for Either<L, R>
        where
            L: Iterator,
            R: Iterator<Item = L::Item>,
        {
            type Item = L::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Either::Left(l) => l.next(),
                    Either::Right(r) => r.next(),
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                match self {
                    Either::Left(l) => l.size_hint(),
                    Either::Right(r) => r.size_hint(),
                }
            }
        }

        match &mut self.0 {
            Flavor::NoThreads(f) => Either::Left(f.fetch_outputs()),
            Flavor::Threads(f) => Either::Right(f.fetch_outputs()),
        }
    }

    /// Retains only the tasks that satisfy the provided predicate.
    pub fn retain_tasks(&mut self, predicate: impl FnMut(&T) -> bool) {
        match &mut self.0 {
            Flavor::NoThreads(f) => f.retain_tasks(predicate),
            Flavor::Threads(f) => f.retain_tasks(predicate),
        }
    }
}

impl<T: Task> Drop for TaskPool<T> {
    fn drop(&mut self) {
        match &self.0 {
            Flavor::NoThreads(_) => (),
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
