use std::ptr::null_mut;

use crate::{
    block::BlockHeader,
    heap::{self, heap_grow},
};

struct Allocator {
    heap_start: *mut u8,
    heap_end: *mut u8,
}
impl Allocator {
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        let mut current = self.heap_start;
        loop {
            let mut head: BlockHeader = BlockHeader::read_from(current);
            if head.size() >= size && !head.is_allocated() {
                head.set_allocated(true); //Marks the block as allocated so that the next call to alloc dosent find the same block and cause a bug
                head.write_to(current);
                return unsafe { current.add(std::mem::size_of::<BlockHeader>()) }; //returns the memory address after the header is complete
            }
            unsafe {
                current = current.add(head.size());
            }
            if current >= self.heap_end {
                return null_mut();
            }
        }
    }
    pub fn dealloc(&mut self, ptr: *mut u8) {
        //Simply marks the block as unallocated
        let mut head =
            BlockHeader::read_from(unsafe { ptr.sub(std::mem::size_of::<BlockHeader>()) });
        head.set_allocated(false);
        head.write_to(unsafe { ptr.sub(std::mem::size_of::<BlockHeader>()) });
    }
}
pub fn init() -> Allocator {
    let temp: usize = 4096; // 4 kb for testng purposes i think lol
    let location = heap_grow(temp);
    let alloc = Allocator {
        heap_start: location,
        heap_end: unsafe { location.add(temp) },
    };
    let header = BlockHeader::new(temp, false);
    unsafe {
        header.write_to(alloc.heap_start);
        header.write_to(alloc.heap_end.sub(std::mem::size_of::<BlockHeader>()));
    }
    alloc
}
