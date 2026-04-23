use std::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
    sync::{LazyLock, Mutex},
};
#[global_allocator]
pub static MY_ALLOCATOR: MyAllocator = MyAllocator; //Static to tell rust which allocator to use
const HEADER_SIZE: usize = std::mem::size_of::<BlockHeader>(); //Constant stores size of one BlockHeader struct
static ALLOCATOR: LazyLock<Mutex<Allocator>> = LazyLock::new(|| Mutex::new(init())); //Global variable to store the Block Pointers
use crate::{
    block::BlockHeader,
    heap::{self, heap_grow},
};
unsafe impl Send for Allocator {} //Tells  the compiler its safe to use for the ALLOCATOR static by
unsafe impl Sync for Allocator {} // Implementing the send and sync trait manually
struct Allocator {
    heap_start: *mut u8,
    heap_end: *mut u8,
}
impl Allocator {
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        let mut current = self.heap_start;
        let size = (size + 7) & !7; //Allign the size before passing it to the loop otherwise it will cause the loop to run stall
        loop {
            let mut head: BlockHeader = BlockHeader::read_from(current);
            if head.size() >= size && !head.is_allocated() {
                head.set_allocated(true); //Marks the block as allocated so that the next call to alloc dosent find the same block and cause a bug
                self.split(current, size); //Split the block if its larger than required to manage memory efficiently
                return unsafe { current.add(HEADER_SIZE) }; //returns the memory address after the header is complete
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
        let mut head = BlockHeader::read_from(unsafe { ptr.sub(HEADER_SIZE) });
        head.set_allocated(false);
        head.write_to(unsafe { ptr.sub(HEADER_SIZE) });
        self.coalesce(unsafe { ptr.sub(HEADER_SIZE) });
    }
    fn coalesce(&mut self, ptr: *mut u8) {
        //Combines 2 free blocks to a single bigger free block
        let current = BlockHeader::read_from(ptr);
        let prev = BlockHeader::read_from(unsafe { ptr.sub(HEADER_SIZE) });
        let next = BlockHeader::read_from(unsafe { ptr.add(current.size()) });
        let mut position = ptr;
        let mut size = current.size();
        if !prev.is_allocated() {
            size += prev.size();
            position = unsafe { ptr.sub(prev.size()) };
        }
        if !next.is_allocated() {
            size += next.size();
        }
        let header = BlockHeader::new(size, false);
        header.write_to(position);
        header.write_to(unsafe { position.add(size - HEADER_SIZE) });
    }
    fn split(&mut self, ptr: *mut u8, size: usize) {
        //
        let header = BlockHeader::read_from(ptr);
        let free_size = header.size() - size;
        if free_size < 2 * HEADER_SIZE + 8 {
            //Check if the remaning storage is big enough to be worth splitting off otherwise return the ptr as is
            return;
        }
        let new_header = BlockHeader::new(size, true);
        new_header.write_to(ptr);
        new_header.write_to(unsafe { ptr.add(size - HEADER_SIZE) });
        let new_header = BlockHeader::new(free_size, false);
        new_header.write_to(unsafe { ptr.add(size) });
        new_header.write_to(unsafe { ptr.add(size + free_size - HEADER_SIZE) });
    }
}

pub struct MyAllocator; //Zero Sized Struct
unsafe impl GlobalAlloc for MyAllocator {
    //Declares the MyAllocator struct as the default memory_allocator instead of system allocator
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut guard = ALLOCATOR.lock().unwrap();
        guard.alloc(layout.size())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        //The trait demands 3 parameters but the 'Layout' is not required here to free the block as the size is already stored in the BlockHeader
        let mut guard = ALLOCATOR.lock().unwrap();
        guard.dealloc(ptr)
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
        header.write_to(alloc.heap_end.sub(HEADER_SIZE));
    }
    alloc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] //simple test function
    fn alloc_split_test() {
        let mut block = init();
        let chunk = block.alloc(40);
        let chunk_header = BlockHeader::read_from(unsafe { chunk.sub(HEADER_SIZE) });
        println!("chunk: {:?}", chunk);
        assert_eq!(40, chunk_header.size());
        assert!(chunk_header.is_allocated());
        let temp1 = block.alloc(8);
        let temp2 = block.alloc(29);
        let temp3 = block.alloc(139);
        block.dealloc(temp2);
        block.dealloc(temp1);
        assert!(!BlockHeader::read_from(unsafe { temp1.sub(HEADER_SIZE) }).is_allocated());
        println!(
            "{}",
            BlockHeader::read_from(unsafe { temp1.sub(HEADER_SIZE) }).size()
        );
    }
}
