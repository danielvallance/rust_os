//! This module provides a thin wrapper around a Future which is the basis of a cooperative
//! multitasking mechanism which this kernel provides.

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub mod keyboard;
pub mod simple_executor;

/// A Task is a thin wrapper around a Future
pub struct Task {
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
            future: Box::pin(future),
        }
    }

    /// Invokes the poll method of the Task's Future
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
