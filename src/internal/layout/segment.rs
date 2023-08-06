use std::mem::size_of;
use std::ptr::NonNull;

use crate::internal::layout::constants::BYTE_BIT_SIZE;
use crate::internal::layout::segment;
use crate::internal::layout::segment_space;
use crate::internal::layout::subheap;
use crate::sys::ptr::AnyNonNullPtr;
use crate::util;

pub const SEGMENT_SIZE: usize = 1 << 16;
pub const COMPACT_HEADER_SIZE: usize = size_of::<CompactHeader>();
const ADDITIONAL_HEADER_SIZE: usize = size_of::<AdditionalHeader>();
pub const BITMAP_ITEM_SIZE: usize = size_of::<usize>();
const BITMAP_ITEM_BIT_SIZE: usize = BITMAP_ITEM_SIZE * BYTE_BIT_SIZE;
const BITMAP_ITEM_SP_BIT_SIZE: usize = 1;
pub const BITMAP_ITEM_EFF_BIT_SIZE: usize = BITMAP_ITEM_BIT_SIZE - BITMAP_ITEM_SP_BIT_SIZE;

#[derive(Clone, Copy)]
pub struct Segment {
    pub compact_header: NonNull<CompactHeader>,
    pub additional_header: NonNull<AdditionalHeader>,
}

impl Segment {
    #[inline]
    pub fn new(compact_header: NonNull<CompactHeader>, segment: AnyNonNullPtr) -> Self {
        Self {
            compact_header: compact_header,
            additional_header: segment.as_nonnull(),
        }
    }

    pub unsafe fn init_single(&mut self, class_of_size: usize) {
        *self.compact_header.as_mut() = CompactHeader {
            next: std::ptr::null_mut(),
            bitmap: 1,
        };
        *self.additional_header.as_mut() = AdditionalHeader {
            prev: std::ptr::null_mut(),
            subheap_class: class_of_size,
            used_block_count: 0,
        };

        let _ = SUB_BITMAP_SIZE_OF_CLASS[class_of_size];
        todo!()
    }

    #[inline]
    pub fn seg_ptr(&self) -> AnyNonNullPtr {
        AnyNonNullPtr::new(self.additional_header)
    }

    #[inline]
    unsafe fn bitmap_space_begin(&self) -> AnyNonNullPtr {
        self.seg_ptr().add(ADDITIONAL_HEADER_SIZE)
    }

    #[inline]
    unsafe fn block_space_begin(&self) -> AnyNonNullPtr {
        let sub_bitmap_size = SUB_BITMAP_SIZE_OF_CLASS[self.subheap_class()];
        self.bitmap_space_begin()
            .add(SUB_BITMAP_UNIT_SIZE * sub_bitmap_size)
    }

    #[inline]
    pub unsafe fn subheap_class(&self) -> usize {
        self.additional_header.as_ref().subheap_class
    }

    #[inline]
    pub unsafe fn block_size(&self) -> usize {
        subheap::SUBHEAP_SIZE_OF_CLASS[self.subheap_class()]
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

    #[inline]
    pub unsafe fn block_ptr(&mut self, index: usize) -> AnyNonNullPtr {
        self.block_space_begin().add(self.block_size() * index)
    }

    #[inline]
    pub unsafe fn from_block_ptr<'a>(
        seg_space: &mut segment_space::SegmentSpace,
        block_ptr: AnyNonNullPtr,
    ) -> (Self, usize) {
        let seg_ptr = AnyNonNullPtr::new(NonNull::new_unchecked(util::bits::max_aligned_size(block_ptr.as_addr(), segment::SEGMENT_SIZE) as *mut ()));
        let seg = seg_space.segment_by_header(seg_ptr);

        let block_space_begin = seg.block_space_begin();
        assert!(block_space_begin <= block_ptr);

        let block_index = (block_ptr.offset_bytes_from(block_space_begin) as usize) / seg.block_size();

        (seg, block_index)
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
    pub used_block_count: usize,
}

const SUB_BITMAP_UNIT_SIZE: usize = BITMAP_ITEM_EFF_BIT_SIZE * BITMAP_ITEM_SIZE;
const SUB_BITMAP_SIZE_OF_CLASS: [usize; subheap::CLASS_COUNT] = [
    sub_bitmap_size_of_class(0),
    sub_bitmap_size_of_class(1),
    sub_bitmap_size_of_class(2),
    sub_bitmap_size_of_class(3),
    sub_bitmap_size_of_class(4),
    sub_bitmap_size_of_class(5),
    sub_bitmap_size_of_class(6),
    sub_bitmap_size_of_class(7),
    sub_bitmap_size_of_class(8),
    sub_bitmap_size_of_class(9),
    sub_bitmap_size_of_class(10),
    sub_bitmap_size_of_class(11),
    sub_bitmap_size_of_class(12),
    sub_bitmap_size_of_class(13),
    sub_bitmap_size_of_class(14),
    sub_bitmap_size_of_class(15),
    sub_bitmap_size_of_class(16),
    sub_bitmap_size_of_class(17),
    sub_bitmap_size_of_class(18),
    sub_bitmap_size_of_class(19),
    sub_bitmap_size_of_class(20),
    sub_bitmap_size_of_class(21),
    sub_bitmap_size_of_class(22),
    sub_bitmap_size_of_class(23),
    sub_bitmap_size_of_class(24),
    sub_bitmap_size_of_class(25),
    sub_bitmap_size_of_class(26),
    sub_bitmap_size_of_class(27),
    sub_bitmap_size_of_class(28),
    sub_bitmap_size_of_class(29),
    sub_bitmap_size_of_class(30),
    sub_bitmap_size_of_class(31),
];

const BLOCK_COUNT_OF_CLASS: [usize; subheap::CLASS_COUNT] = [
    block_count_of_class(0),
    block_count_of_class(1),
    block_count_of_class(2),
    block_count_of_class(3),
    block_count_of_class(4),
    block_count_of_class(5),
    block_count_of_class(6),
    block_count_of_class(7),
    block_count_of_class(8),
    block_count_of_class(9),
    block_count_of_class(10),
    block_count_of_class(11),
    block_count_of_class(12),
    block_count_of_class(13),
    block_count_of_class(14),
    block_count_of_class(15),
    block_count_of_class(16),
    block_count_of_class(17),
    block_count_of_class(18),
    block_count_of_class(19),
    block_count_of_class(20),
    block_count_of_class(21),
    block_count_of_class(22),
    block_count_of_class(23),
    block_count_of_class(24),
    block_count_of_class(25),
    block_count_of_class(26),
    block_count_of_class(27),
    block_count_of_class(28),
    block_count_of_class(29),
    block_count_of_class(30),
    block_count_of_class(31),
];

const fn sub_bitmap_size_of_class(class_of_size: usize) -> usize {
    let block_size = subheap::SUBHEAP_SIZE_OF_CLASS[class_of_size];
    let block_space_size = SEGMENT_SIZE - size_of::<AdditionalHeader>();

    if block_space_size <= block_size * BITMAP_ITEM_EFF_BIT_SIZE {
        0
    } else {
        ((block_space_size - 1)
            / ((block_size * BITMAP_ITEM_EFF_BIT_SIZE + BITMAP_ITEM_SIZE)
                * BITMAP_ITEM_EFF_BIT_SIZE))
            + 1
    }
}

const fn block_count_of_class(class_of_size: usize) -> usize {
    let block_size = subheap::SUBHEAP_SIZE_OF_CLASS[class_of_size];
    let sub_bitmap_size = sub_bitmap_size_of_class(class_of_size);
    let segment_available_size = SEGMENT_SIZE - ADDITIONAL_HEADER_SIZE;
    let block_space_size = segment_available_size - SUB_BITMAP_UNIT_SIZE * sub_bitmap_size;

    block_space_size / block_size
}
