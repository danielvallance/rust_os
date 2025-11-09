//! This module provides a thin wrapper around a Future which is the basis of a cooperative
//! multitasking mechanism which this kernel provides.

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

/// Identifier for Task instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        // Initialise the NEXT_ID static variable as 0 only once
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        // Atomically fetch and add NEXT_ID to get a guaranteed unique ID
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A Task is a thin wrapper around a Future
pub struct Task {
    id: TaskId,

    /// The Task has a reference to a Future which has no
    /// return value (it is just executed for its side effects)
    ///
    /// The Future is wrapped in a Box which refers to a dynamically
    /// dispatched Future, which means it can refer to any async function
    ///
    /// Finally the Box is wrapped in a Pin so if there is a self-referential
    /// struct in the state of the Future, undefined behaviour will not be incurred
    /// by copying it around in memory as it is 'Pinned' to a single location.
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Creates a new Task by passing it an async function
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// Invokes the poll method of the Task's Future
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
