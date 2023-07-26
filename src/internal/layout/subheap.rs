use std::ptr::NonNull;

use crate::internal::layout::constants::ALIGNMENT_SIZE;
use crate::internal::layout::segment;
use crate::util;

pub const CLASS_COUNT: usize = 32;

#[derive(Debug)]
pub struct SubHeap {
    pub free_segments_begin: *mut segment::CompactHeader,
    pub free_segments_end: *mut segment::CompactHeader,
}

impl SubHeap {
    pub fn init() -> Self {
        Self {
            free_segments_begin: std::ptr::null_mut(),
            free_segments_end: std::ptr::null_mut(),
        }
    }

    pub fn next_free_segment(&self) -> Option<NonNull<segment::CompactHeader>> {
        NonNull::new(self.free_segments_begin)
    }
}

pub const SUBHEAP_SIZE_OF_CLASS: [usize; CLASS_COUNT] = [
    // 0-3
    0x0001 * ALIGNMENT_SIZE,
    0x0002 * ALIGNMENT_SIZE,
    0x0003 * ALIGNMENT_SIZE,
    0x0004 * ALIGNMENT_SIZE,
    // 4-9
    0x0006 * ALIGNMENT_SIZE,
    0x0008 * ALIGNMENT_SIZE,
    0x000a * ALIGNMENT_SIZE,
    0x000c * ALIGNMENT_SIZE,
    0x000e * ALIGNMENT_SIZE,
    0x0010 * ALIGNMENT_SIZE,
    // 10-24
    0x0020 * ALIGNMENT_SIZE,
    0x0030 * ALIGNMENT_SIZE,
    0x0040 * ALIGNMENT_SIZE,
    0x0050 * ALIGNMENT_SIZE,
    0x0060 * ALIGNMENT_SIZE,
    0x0070 * ALIGNMENT_SIZE,
    0x0080 * ALIGNMENT_SIZE,
    0x0090 * ALIGNMENT_SIZE,
    0x00a0 * ALIGNMENT_SIZE,
    0x00b0 * ALIGNMENT_SIZE,
    0x00c0 * ALIGNMENT_SIZE,
    0x00d0 * ALIGNMENT_SIZE,
    0x00e0 * ALIGNMENT_SIZE,
    0x00f0 * ALIGNMENT_SIZE,
    0x0100 * ALIGNMENT_SIZE,
    // 25-31
    0x0200 * ALIGNMENT_SIZE,
    0x0300 * ALIGNMENT_SIZE,
    0x0400 * ALIGNMENT_SIZE,
    0x0500 * ALIGNMENT_SIZE,
    0x0600 * ALIGNMENT_SIZE,
    0x0700 * ALIGNMENT_SIZE,
    0x0800 * ALIGNMENT_SIZE,
];

pub const fn class_of_size(size: usize) -> Option<usize> {
    assert!(util::bits::is_aligned(size, ALIGNMENT_SIZE));

    let align_size = size / ALIGNMENT_SIZE;

    if align_size > 0x800 {
        None
    } else if align_size > 0x100 {
        let i = (align_size - 1) / 0x100;
        Some(24 + i)
    } else if align_size > 0x10 {
        let i = (align_size - 1) / 0x10;
        Some(9 + i)
    } else if align_size > 0x4 {
        let i = (align_size - 1) / 0x2;
        Some(2 + i)
    } else {
        Some(align_size - 1)
    }
}
