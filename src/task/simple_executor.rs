//! This module maintains a queue of Tasks and continuously polls them,
//! adding them back to the Task queue if they have not completed.

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// The SimpleExecutor maintains a queue of Tasks
pub struct SimpleExecutor {
    /// A queue of Tasks
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    /// Obtain a new SimpleExecutor with an empty Task queue
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    /// Add a task to the SimpleExecutor's Task queue
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    /// Continuously poll the Tasks queue while there are still Tasks to be completed
    pub fn run(&mut self) {
        // Get next unfinished Task in the queue
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);

            // Add the Task back to the queue if it has not completed
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

impl Default for SimpleExecutor {
    fn default() -> Self {
        SimpleExecutor::new()
    }
}

/// Creates a RawWaker which does nothing
fn dummy_raw_waker() -> RawWaker {
    /// Takes a constant raw pointer and does nothing. This is the type signature
    /// of the waker methods defined in the RawWakerVTable
    fn no_op(_: *const ()) {}

    /// Returns another RawWaker
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    // Creates a RawWakerVTable which defines a set of callbacks. It triggers 'clone' on a clone event, and 'no_op' on wake and drop events.
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);

    // Returns a new RawWaker
    RawWaker::new(core::ptr::null::<()>(), vtable)
}

/// Creates a dummy Waker
fn dummy_waker() -> Waker {
    // The dummy Waker is created from a RawWaker as that is the only way to define a Waker which does nothing
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
