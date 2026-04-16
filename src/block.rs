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
}
