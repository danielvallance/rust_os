//! This module provides support for asynchronously processing key presses by reading them from a
//! queue which does not block or allocate on push/pop operations. It makes use of Waker notifications
//! so the executor does not have to continuously poll the Task. Once the key has been read, it gets printed
//! to the VGA buffer.

use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

/// Queue for scancodes which does not block or allocate on push/pop operations
///
/// It is wrapped in a OnceCell to allow a safe, one time initialisation
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// Global waker which will notify the executor when the print_keypresses Task can make progress.
///
/// Since it uses atomic operations, it is safe to have it as a static variable
static WAKER: AtomicWaker = AtomicWaker::new();

/// Struct which implements the Stream trait for asynchronously returning keypresses
/// from the queue.
pub struct ScancodeStream {
    /// Private member whose sole purpose is to prevent other modules from instantiating it
    /// without using the new() method.
    _private: (),
}

impl ScancodeStream {
    /// Initialise the scancode queue with a bounded capacity of 100 (to prevent any allocations)
    /// and return an instance of the ScancodeStream struct
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> ScancodeStream {
        Self::new()
    }
}

impl Stream for ScancodeStream {
    /// This Stream returns keypresses
    type Item = u8;

    /// Poll the queue for any recent keypresses
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        // Register the Waker in case this returns Poll::Pending, and we
        // want to obtain a handle with which WAKER can wake up the executor
        // when a key is later added to the queue.
        WAKER.register(cx.waker());
        match queue.pop() {
            Some(scancode) => {
                // Discard the waker if a key press has since entered the queue,
                // as this call will not return Poll::Pending
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate as doing so could cause a deadlock.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // Notify the executor that a key press has entered the queue, so poll the print_keypresses Task
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// Takes keypresses from the queue and prints them to the VGA buffer
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    // Keyboard representation which is instantiated with scancode set 1,
    // US layout, and its behaviour of handling 'ctrl' combinations like normal keys
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    // Convert scancode to an Option<KeyEvent> which represents the key in question, and if it was a key up or down event.
    // Then convert the key into a character, and print it
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode)
            && let Some(key) = keyboard.process_keyevent(key_event)
        {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
}
