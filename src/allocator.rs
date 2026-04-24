use std::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
    sync::{LazyLock, Mutex},
};

const HEADER_SIZE: usize = std::mem::size_of::<BlockHeader>(); //Constant stores size of one BlockHeader struct
static ALLOCATOR: LazyLock<Mutex<Allocator>> = LazyLock::new(|| Mutex::new(init())); //Global variable to store the Block Pointers
use crate::{
    block::{BlockHeader, FreeHeader},
    heap::{self, heap_grow},
};
unsafe impl Send for Allocator {} //Tells  the compiler its safe to use for the ALLOCATOR static by
unsafe impl Sync for Allocator {} // Implementing the send and sync trait manually
struct Allocator {
    heap_start: *mut u8,
    heap_end: *mut u8,
    free_head_start: *mut u8,
}
impl Allocator {
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        let mut current = self.free_head_start;
        let size = (size + 7) & !7; //Allign the size before passing it to the loop otherwise it will cause the loop to run stall
        loop {
            let mut head: BlockHeader = BlockHeader::read_from(current);
            if head.size() >= size && !head.is_allocated() {
                self.remove_free(current);
                head.set_allocated(true); //Marks the block as allocated so that the next call to alloc dosent find the same block and cause a bug
                head.write_to(current);
                self.split(current, size); //Split the block if its larger than required to manage memory efficiently
                return unsafe { current.add(HEADER_SIZE) }; //returns the memory address after the header is complete
            }
            unsafe {
                let next = FreeHeader::read_from(current.add(HEADER_SIZE)).get_next();
                if next.is_null() {
                    return std::ptr::null_mut();
                }
                current = next;
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
        let next_ptr = unsafe { ptr.add(current.size()) };
        let mut position = ptr;
        let mut size = current.size();
        if !prev.is_allocated() {
            size += prev.size();
            position = unsafe { ptr.sub(prev.size()) };
            self.remove_free(position);
        }
        if !next.is_allocated() {
            size += next.size();
            self.remove_free(next_ptr);
        }
        let header = BlockHeader::new(size, false);
        header.write_to(position);
        header.write_to(unsafe { position.add(size - HEADER_SIZE) });
        self.insert_free(position);
    }
    fn split(&mut self, ptr: *mut u8, size: usize) {
        let header = BlockHeader::read_from(ptr);
        let free_size = header.size() - size;
        if free_size < 2 * HEADER_SIZE + std::mem::size_of::<FreeHeader>() {
            //Check if the remaning storage is big enough to be worth splitting off otherwise return the ptr as is
            return;
        }
        let new_header = BlockHeader::new(size, true);
        new_header.write_to(ptr);
        new_header.write_to(unsafe { ptr.add(size - HEADER_SIZE) });
        let new_header = BlockHeader::new(free_size, false);
        new_header.write_to(unsafe { ptr.add(size) });
        new_header.write_to(unsafe { ptr.add(size + free_size - HEADER_SIZE) });
        self.insert_free(unsafe { ptr.add(size) });
    }
    fn insert_free(&mut self, ptr: *mut u8) {
        //Used to insert a pointer to a newly freed block to an old block or create a new 'free_head_start' if none exists
        if self.free_head_start.is_null() {
            let free_header = FreeHeader::new(std::ptr::null_mut(), std::ptr::null_mut());
            free_header.write_to(unsafe { ptr.add(HEADER_SIZE) });
            self.free_head_start = ptr;
            return;
        }
        let free_header = FreeHeader::new(std::ptr::null_mut(), self.free_head_start);
        let mut new_header =
            FreeHeader::read_from(unsafe { self.free_head_start.add(HEADER_SIZE) });
        new_header.set_prev(ptr);
        free_header.write_to(unsafe { ptr.add(HEADER_SIZE) });
        new_header.write_to(unsafe { self.free_head_start.add(HEADER_SIZE) });
        self.free_head_start = ptr;
    }
    fn remove_free(&mut self, ptr: *mut u8) {
        //This is used to remove free block pointers which have been filled or merged to one through coalesce
        //Handles 4 cases : If prev is null , if next is null , if both are null , if both esist
        if !self.free_head_start.is_null() {
            let header = FreeHeader::read_from(unsafe { ptr.add(HEADER_SIZE) });
            let prev_ptr = header.get_prev();
            let next_ptr = header.get_next();
            if prev_ptr.is_null() {
                if next_ptr.is_null() {
                    self.free_head_start = std::ptr::null_mut();
                    return;
                }
                self.free_head_start = next_ptr;
                let mut new_header = FreeHeader::read_from(unsafe { next_ptr.add(HEADER_SIZE) });
                new_header.set_prev(std::ptr::null_mut());
                new_header.write_to(unsafe { next_ptr.add(HEADER_SIZE) });
                return;
            }
            if next_ptr.is_null() {
                if prev_ptr.is_null() {
                    self.free_head_start = std::ptr::null_mut();
                    return;
                }
                let mut header = FreeHeader::read_from(unsafe { prev_ptr.add(HEADER_SIZE) });
                header.set_next(std::ptr::null_mut());
                header.write_to(unsafe { prev_ptr.add(HEADER_SIZE) });
                return;
            }
            let mut prev_header = FreeHeader::read_from(unsafe { prev_ptr.add(HEADER_SIZE) });
            prev_header.set_next(next_ptr);
            let mut next_header = FreeHeader::read_from(unsafe { next_ptr.add(HEADER_SIZE) });
            next_header.set_prev(prev_ptr);
            prev_header.write_to(unsafe { prev_ptr.add(HEADER_SIZE) });
            next_header.write_to(unsafe { next_ptr.add(HEADER_SIZE) });
        }
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
    let temp: usize = 1024 * 1024; // 1 MB for testng purposes
    let location = heap_grow(temp);
    let alloc = Allocator {
        heap_start: location,
        heap_end: unsafe { location.add(temp) },
        free_head_start: location,
    };
    let header = BlockHeader::new(temp, false);
    unsafe {
        header.write_to(alloc.heap_start);
        header.write_to(alloc.heap_end.sub(HEADER_SIZE));
    }
    FreeHeader::new(std::ptr::null_mut(), std::ptr::null_mut())
        .write_to(unsafe { location.add(HEADER_SIZE) });
    alloc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] //simple test function
    fn alloc_split_test() {
        let mut block = init();
        println!("init done");
        let chunk = block.alloc(40);
        println!("alloc 40: {:?}", chunk);
        let chunk_header = BlockHeader::read_from(unsafe { chunk.sub(HEADER_SIZE) });
        println!("chunk: {:?}", chunk);
        assert_eq!(40, chunk_header.size());
        assert!(chunk_header.is_allocated());
        let temp1 = block.alloc(8);
        println!("alloc 8: {:?}", temp1);
        let temp2 = block.alloc(29);
        println!("alloc 29: {:?}", temp2);
        let temp3 = block.alloc(139);
        println!("alloc 139: {:?}", temp3);
        block.dealloc(temp2);
        block.dealloc(temp1);
        assert!(!BlockHeader::read_from(unsafe { temp1.sub(HEADER_SIZE) }).is_allocated());
        println!(
            "{}",
            BlockHeader::read_from(unsafe { temp1.sub(HEADER_SIZE) }).size()
        );
    }
}
