use std::mem::size_of;

pub const SEGMENT_SIZE: usize = 1 << 16;
pub const COMPACT_HEADER_SIZE: usize = size_of::<CompactHeader>();

pub struct CompactHeader {
    pub bitmap: usize,
    pub raw_next_addr_with_flags: usize,
}

impl CompactHeader {
    pub fn next(&self) -> *mut CompactHeader {
        ((self.raw_next_addr_with_flags / 4) * 4) as *mut CompactHeader
    }

    pub fn is_committed(&self) -> bool {
        self.raw_next_addr_with_flags & 0b1 == 0b1
    }

    pub fn is_soft_decommitted(&self) -> bool {
        self.raw_next_addr_with_flags & 0b10 == 0b10
    }
}

pub struct Header {
    pub prev: *mut Header,
    pub subheap_class: isize,
}