//! This module contains functions which deal with paging and memory allocation

use x86_64::{VirtAddr, structures::paging::PageTable};

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // The address in CR3 contains the physical address of the active level 4 page table
    let (level_4_table_frame, _) = Cr3::read();

    // Find the address of the active level 4 page table in virtual memory by adding the physical memory offset.
    // This works because the entirety of physical memory is mapped to virtual memory,
    // starting at the physical memory offset.
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
