#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator;

pub static mut HEAP_START: usize = 0;
pub static mut HEAP_SIZE: usize = 0;
pub static mut NEXT: usize = 0;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc_start = (HEAP_START + NEXT + layout.align() - 1) & !(layout.align() - 1);
        let alloc_end = alloc_start.checked_add(layout.size()).expect("overflow");

        if alloc_end > HEAP_START + HEAP_SIZE {
            ptr::null_mut() // Out of memory
        } else {
            NEXT = alloc_end - HEAP_START;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator does not support deallocation
    }
}

pub fn init_heap(start: usize, size: usize) {
    unsafe {
        HEAP_START = start;
        HEAP_SIZE = size;
        NEXT = 0;
    }
}
