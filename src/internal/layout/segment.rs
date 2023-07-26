use std::mem::size_of;
use std::ptr::NonNull;

use crate::sys::ptr::AnyNonNullPtr;

pub const SEGMENT_SIZE: usize = 1 << 16;
pub const COMPACT_HEADER_SIZE: usize = size_of::<CompactHeader>();

#[derive(Clone, Copy)]
pub struct Segment {
    pub raw_compact_header: NonNull<CompactHeader>,
    pub raw_additional_header: NonNull<AdditionalHeader>,
}

impl Segment {
    pub unsafe fn init_single_committed(&mut self, block_size: usize) {
        *self.raw_compact_header.as_mut() = CompactHeader {
            bitmap: 0,
            raw_next_addr_with_flags: 0b1,
        };
        *self.raw_additional_header.as_mut() = AdditionalHeader {
            prev: std::ptr::null_mut(),
            subheap_class: 0,
        };
        todo!("initialize sub-bitmaps and blocks")
    }

    pub fn from_block_ptr<'a>(ptr: AnyNonNullPtr) -> (&'a mut Self, usize) {
        todo!()
    }

    #[inline]
    pub fn pointer_to_seg(&self) -> *mut CompactHeader {
        self.raw_compact_header.as_ptr()
    }

    #[inline]
    pub unsafe fn subheap_class(&self) -> usize {
        self.raw_additional_header.as_ref().subheap_class
    }

    #[inline]
    pub unsafe fn next(&self) -> *mut CompactHeader {
        self.raw_compact_header.as_ref().next()
    }

    #[inline]
    pub unsafe fn set_next(&mut self, ptr: *mut CompactHeader) {
        self.raw_compact_header.as_mut().set_next(ptr)
    }

    #[inline]
    pub unsafe fn prev(&self) -> *mut CompactHeader {
        self.raw_additional_header.as_ref().prev
    }

    #[inline]
    pub unsafe fn set_prev(&mut self, ptr: *mut CompactHeader) {
        self.raw_additional_header.as_mut().prev = ptr;
    }

    pub fn block_ptr(&mut self, index: usize) -> AnyNonNullPtr {
        todo!()
    }

    #[inline]
    pub unsafe fn is_floated(&self) -> bool {
        self.raw_additional_header.as_ref().prev.is_null()
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

        self.raw_compact_header
            .as_mut()
            .set_next(after.pointer_to_seg());
        after.raw_additional_header.as_mut().prev = self.pointer_to_seg();
    }
}

pub struct CompactHeader {
    pub bitmap: usize,
    pub raw_next_addr_with_flags: usize,
}

impl CompactHeader {
    #[inline]
    pub fn next(&self) -> *mut Self {
        (self.raw_next_addr_with_flags & !0b11) as *mut Self
    }

    #[inline]
    pub fn set_next(&mut self, ptr: *mut Self) {
        let addr = ptr as usize;
        assert!(addr & 0b11 == 0);

        let flags = self.raw_next_addr_with_flags & 0b11;
        self.raw_next_addr_with_flags = addr + flags;
    }

    #[allow(unused)]
    #[inline]
    pub fn is_committed(&self) -> bool {
        self.raw_next_addr_with_flags & 0b1 == 0b1
    }

    pub fn set_committed(&mut self, committed: bool) {
        if committed {
            self.raw_next_addr_with_flags = self.raw_next_addr_with_flags | 0b1;
        } else {
            self.raw_next_addr_with_flags = self.raw_next_addr_with_flags & !0b1;
        }
    }
}

pub struct AdditionalHeader {
    pub prev: *mut CompactHeader,
    pub subheap_class: usize,
}
