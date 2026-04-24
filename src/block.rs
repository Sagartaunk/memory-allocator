use std::ptr;
#[derive(Clone, Copy)]
pub struct BlockHeader {
    size: usize,
}
impl BlockHeader {
    pub fn new(size: usize, allocated: bool) -> Self {
        let size_part = (size + 7) & !7;
        debug_assert!(size_part % 8 == 0);
        let flag_part = allocated as usize;
        BlockHeader {
            size: size_part | flag_part,
        }
    }
    pub fn size(&self) -> usize {
        self.size & !1
    }
    pub fn is_allocated(&self) -> bool {
        (self.size & 1) == 1
    }
    pub fn set_allocated(&mut self, change: bool) {
        if change {
            self.size |= 1;
        } else {
            self.size &= !1;
        }
    }
    pub fn read_from(location: *const u8) -> Self {
        unsafe {
            let header = location as *const BlockHeader;
            ptr::read(header)
        }
    }
    pub fn write_to(&self, location: *mut u8) {
        unsafe {
            let header = location as *mut BlockHeader;
            ptr::write(header, *self);
        }
    }
}

#[derive(Clone, Copy)]
pub struct FreeHeader {
    prev: *mut u8,
    next: *mut u8,
}
impl FreeHeader {
    pub fn new(prev: *mut u8, next: *mut u8) -> Self {
        FreeHeader { prev, next }
    }
    pub fn read_from(location: *const u8) -> Self {
        unsafe {
            let header = location as *const FreeHeader;
            ptr::read(header)
        }
    }
    pub fn write_to(&self, location: *mut u8) {
        unsafe {
            let header = location as *mut FreeHeader;
            ptr::write(header, *self);
        }
    }
    pub fn set_prev(&mut self, prev: *mut u8) {
        self.prev = prev;
    }
    pub fn set_next(&mut self, next: *mut u8) {
        self.next = next;
    }
    pub fn get_next(&self) -> *mut u8 {
        self.next
    }
    pub fn get_prev(&self) -> *mut u8 {
        self.prev
    }
}
