//! This module provides a data type which implements the GlobalAlloc trait for use by the kernel

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

/// Starting address of heap region in virtual memory
pub const HEAP_START: usize = 0x_4444_4444_0000;

/// Size of heap (100 KiB)
pub const HEAP_SIZE: usize = 100 * 1024;

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

/// Initialises heap by allocating frames of physical memory,
/// and mapping pages in the heap region to them
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Iterate over pages in the heap range, and for each one map it
    // to a physical frame. This physical frame must be allocated first.
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    Ok(())
}
