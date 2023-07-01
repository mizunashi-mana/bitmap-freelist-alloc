use crate::internal::constants::ALIGNMENT_SIZE;
use crate::util;

pub struct HeaderForFreeSize {
    block_size_with_flags: usize,
}

impl HeaderForFreeSize {
    pub unsafe fn init(p: *mut HeaderForFreeSize, block_size: usize) {
        assert!(util::bits::is_aligned(block_size, ALIGNMENT_SIZE));

        *p = HeaderForFreeSize {
            block_size_with_flags: block_size,
        };
    }

    pub fn block_size(&self) -> usize {
        util::bits::max_aligned_size(self.block_size_with_flags, ALIGNMENT_SIZE)
    }
}
