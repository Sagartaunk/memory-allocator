use libc::{MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE, mmap};
use std::ptr;
pub fn heap_grow(size: usize) -> *mut u8 {
    unsafe {
        let ptr = mmap(
            ptr::null_mut(), // asks to get allocation wherever free from the os
            size,            // size in bytes
            PROT_READ | PROT_WRITE,
            MAP_ANONYMOUS | MAP_PRIVATE,
            -1,
            0,
        );
        if ptr == MAP_FAILED {
            ptr::null_mut()
        } else {
            ptr as *mut u8
        }
    }
}
