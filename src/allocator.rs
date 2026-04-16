use crate::{
    block::BlockHeader,
    heap::{self, heap_grow},
};

struct Allocator {
    heap_start: *mut u8,
    heap_end: *mut u8,
}
pub fn init() -> Allocator {
    let temp: usize = 4096; // 4 mb for testng purposes i think lol
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
