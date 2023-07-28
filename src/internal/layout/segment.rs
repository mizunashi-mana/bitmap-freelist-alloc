use std::mem::size_of;
use std::ptr::NonNull;

use crate::internal::layout::segment;
use crate::internal::layout::segment_space;
use crate::sys::ptr::AnyNonNullPtr;
use crate::util;

pub const SEGMENT_SIZE: usize = 1 << 16;
pub const COMPACT_HEADER_SIZE: usize = size_of::<CompactHeader>();

#[derive(Clone, Copy)]
pub struct Segment {
    pub compact_header: NonNull<CompactHeader>,
    pub additional_header: NonNull<AdditionalHeader>,
}

impl Segment {
    pub fn new(compact_header: NonNull<CompactHeader>, segment: AnyNonNullPtr) -> Self {
        Self {
            compact_header: compact_header,
            additional_header: segment.as_nonnull(),
        }
    }

    pub unsafe fn init_single(&mut self, block_size: usize) {
        *self.compact_header.as_mut() = CompactHeader {
            next: std::ptr::null_mut(),
            bitmap: 0,
        };
        *self.additional_header.as_mut() = AdditionalHeader {
            prev: std::ptr::null_mut(),
            subheap_class: 0,
        };
        todo!("initialize sub-bitmaps and blocks")
    }

    pub fn from_block_ptr<'a>(
        seg_space: &mut segment_space::SegmentSpace,
        ptr: AnyNonNullPtr,
    ) -> (Self, usize) {
        let _ = util::bits::max_aligned_size(ptr.as_addr(), segment::SEGMENT_SIZE);
        todo!()
    }

    #[inline]
    pub fn seg_ptr(&self) -> AnyNonNullPtr {
        AnyNonNullPtr::new(self.additional_header)
    }

    #[inline]
    pub unsafe fn subheap_class(&self) -> usize {
        self.additional_header.as_ref().subheap_class
    }

    #[inline]
    pub unsafe fn next(&self) -> *mut CompactHeader {
        self.compact_header.as_ref().next
    }

    #[inline]
    pub unsafe fn set_next(&mut self, ptr: *mut CompactHeader) {
        self.compact_header.as_mut().next = ptr;
    }

    #[inline]
    pub unsafe fn prev(&self) -> *mut CompactHeader {
        self.additional_header.as_ref().prev
    }

    #[inline]
    pub unsafe fn set_prev(&mut self, ptr: *mut CompactHeader) {
        self.additional_header.as_mut().prev = ptr;
    }

    pub fn block_ptr(&mut self, index: usize) -> AnyNonNullPtr {
        todo!()
    }

    #[inline]
    pub unsafe fn is_floated(&self) -> bool {
        self.additional_header.as_ref().prev.is_null()
            && self.compact_header.as_ref().next.is_null()
    }

    pub fn find_free_block(&mut self) -> Option<usize> {
        todo!()
    }

    pub fn mark_block_and_check_full(&mut self, index: usize) -> bool {
        todo!()
    }

    pub fn free_block_and_check_empty(&mut self, index: usize) -> bool {
        todo!()
    }

    #[inline]
    pub unsafe fn append(&mut self, after: &mut Self) {
        assert!(self.next().is_null());
        assert!(after.prev().is_null());

        self.compact_header.as_mut().next = after.compact_header.as_ptr();
        after.additional_header.as_mut().prev = self.compact_header.as_ptr();
    }
}

pub struct CompactHeader {
    pub next: *mut CompactHeader,
    pub bitmap: usize,
}

pub struct AdditionalHeader {
    pub prev: *mut CompactHeader,
    pub subheap_class: usize,
}
