//! This module provides a data type which implements the GlobalAlloc trait for use by the kernel

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// This struct provides a bare minimum implementation of the GlobalAlloc trait
pub struct Dummy;

// This attribute tells the Rust compiler that ALLOCATOR should be used as the heap allocator
#[global_allocator]
static ALLOCATOR: Dummy = Dummy;

unsafe impl GlobalAlloc for Dummy {
    /// Dummy does not perform allocation, it simply returns a null pointer
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    /// Dummy panics on deallocation
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!(
            "dealloc should never be called, as Dummy does not have a functional alloc implementation"
        )
    }
}
