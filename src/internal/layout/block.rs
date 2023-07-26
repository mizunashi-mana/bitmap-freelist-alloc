use std::ptr::NonNull;

use crate::internal::layout::constants::ALIGNMENT_SIZE;
use crate::util;

pub enum Type {
    FreeSize,
    OnSubHeap,
}

pub struct HeaderForFreeSize {
    block_size_with_flags: usize,
}

impl HeaderForFreeSize {
    pub unsafe fn init(mut ptr: NonNull<HeaderForFreeSize>, block_size: usize) {
        assert!(util::bits::is_aligned(block_size, ALIGNMENT_SIZE));

        *ptr.as_mut() = HeaderForFreeSize {
            block_size_with_flags: block_size,
        };
    }

    pub fn block_size(&self) -> usize {
        util::bits::max_aligned_size(self.block_size_with_flags, ALIGNMENT_SIZE)
    }
}
