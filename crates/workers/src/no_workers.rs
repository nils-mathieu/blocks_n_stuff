use std::cell::RefCell;
use std::collections::BinaryHeap;

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

struct State<W: Worker> {
    tasks: BinaryHeap<Task<W::Input>>,
    worker: Option<W>,
}

pub struct TaskPool<W: Worker>(RefCell<State<W>>);

impl<W: Worker> Default for TaskPool<W> {
    #[inline]
    fn default() -> Self {
        Self(RefCell::new(State {
            tasks: BinaryHeap::new(),
            worker: None,
        }))
    }
}

/// Public methods that will be called by the main implementation.
impl<W: Worker> TaskPool<W> {
    #[inline]
    pub fn task_count(&self) -> usize {
        self.0.borrow().tasks.len()
    }

    pub fn submit(&self, payload: W::Input, priority: Priority) {
        self.0.borrow_mut().tasks.push(Task { payload, priority });
    }

    pub fn submit_batch(&self, iter: impl IntoIterator<Item = (W::Input, Priority)>) {
        self.0.borrow_mut().tasks.extend(
            iter.into_iter()
                .map(|(payload, priority)| Task { payload, priority }),
        );
    }

    pub fn fetch_results(&self) -> impl Iterator<Item = W::Output> + '_ {
        let mut guard = self.0.borrow_mut();
        let guard = &mut *guard;

        if let Some(worker) = guard.worker.as_mut() {
            if let Some(task) = guard.tasks.pop() {
                return Some(worker.run(task.payload)).into_iter();
            }
        }

        None.into_iter()
    }

    pub fn spawn(&self, worker: W) {
        self.0.borrow_mut().worker = Some(worker);
    }
}
