//! This module provides a data type which implements the GlobalAlloc trait for use by the kernel

use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

use crate::allocator::bump::{BumpAllocator, Locked};

pub mod bump;

/// Starting address of heap region in virtual memory
pub const HEAP_START: usize = 0x_4444_4444_0000;

/// Size of heap (100 KiB)
pub const HEAP_SIZE: usize = 100 * 1024;

// This attribute tells the Rust compiler that ALLOCATOR should be used as the heap allocator
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

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

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
