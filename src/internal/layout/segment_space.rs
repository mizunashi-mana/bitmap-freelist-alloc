use std::error::Error;
use std::ptr::NonNull;

use crate::internal::layout::segment;
use crate::sys::ptr::AnyNonNullPtr;
use crate::sys::SysMemEnv;

#[derive(Debug)]
pub struct SegmentSpace {
    // immutable
    pub page_size: usize,
    segment_compact_header_space: AnyNonNullPtr,
    segment_space_begin: AnyNonNullPtr,
    segment_space_end: AnyNonNullPtr,

    // mutable
    pub available_size: usize,
    next_alloc_segment_compact_header_index: usize,
    next_alloc_segment_index: usize,
}

impl SegmentSpace {
    pub fn new(
        page_size: usize,
        segment_compact_header_space: AnyNonNullPtr,
        segment_space_begin: AnyNonNullPtr,
        segment_space_end: AnyNonNullPtr,
        available_size: usize,
        next_alloc_segment_compact_header_index: usize,
        next_alloc_segment_index: usize,
    ) -> Self {
        Self {
            page_size,
            segment_compact_header_space,
            segment_space_begin,
            segment_space_end,
            available_size,
            next_alloc_segment_compact_header_index,
            next_alloc_segment_index,
        }
    }

    pub unsafe fn ptr_in_space(&self, ptr: AnyNonNullPtr) -> bool {
        ptr.offset_bytes_from(self.segment_space_begin) >= 0
            && self.segment_space_end.offset_bytes_from(ptr) > 0
    }

    pub unsafe fn segment(&self, seg_ptr: NonNull<segment::CompactHeader>) -> segment::Segment {
        let raw_seg_ptr = AnyNonNullPtr::new(seg_ptr);
        let seg_index = (raw_seg_ptr.offset_bytes_from(self.segment_compact_header_space) as usize)
            / segment::COMPACT_HEADER_SIZE;
        assert!(seg_index < self.next_alloc_segment_index);

        let raw_additional_header = self
            .segment_space_begin
            .add(seg_index * segment::SEGMENT_SIZE);

        segment::Segment::new(seg_ptr, raw_additional_header)
    }

    pub unsafe fn is_last_segment(&self, seg: segment::Segment) -> bool {
        let seg_index = (seg.seg_ptr().offset_bytes_from(self.segment_space_begin) as usize)
            / segment::SEGMENT_SIZE;
        assert!(seg_index < self.next_alloc_segment_index);

        seg_index == self.next_alloc_segment_index - 1
    }

    pub unsafe fn alloc_new_segment<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
    ) -> Result<Option<segment::Segment>, Box<dyn Error>> {
        if self.next_alloc_segment_compact_header_index == self.next_alloc_segment_index {
            if !self.alloc_new_segment_compact_headers(env)? {
                return Ok(None);
            }
        }

        if self.available_size < segment::SEGMENT_SIZE {
            return Ok(None);
        }

        let next_alloc_segment_index = self.next_alloc_segment_index;
        let new_segment_compact_header_space_begin = self
            .segment_compact_header_space
            .add(next_alloc_segment_index * segment::COMPACT_HEADER_SIZE);
        let new_segment_space_begin = self
            .segment_space_begin
            .add(next_alloc_segment_index * segment::SEGMENT_SIZE);
        env.commit(new_segment_space_begin, segment::SEGMENT_SIZE)?;
        self.available_size -= segment::SEGMENT_SIZE;

        self.next_alloc_segment_index += 1;

        Ok(Some(segment::Segment::new(
            new_segment_compact_header_space_begin.as_nonnull(),
            new_segment_space_begin,
        )))
    }

    unsafe fn alloc_new_segment_compact_headers<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
    ) -> Result<bool, Box<dyn Error>> {
        if self.available_size < self.page_size {
            return Ok(false);
        }

        let new_segment_compact_header_space_begin = self
            .segment_compact_header_space
            .add(self.next_alloc_segment_compact_header_index * segment::COMPACT_HEADER_SIZE);
        let new_segment_compact_headers_count = self.page_size / segment::COMPACT_HEADER_SIZE;
        env.commit(new_segment_compact_header_space_begin, self.page_size)?;
        self.available_size -= self.page_size;

        self.next_alloc_segment_compact_header_index += new_segment_compact_headers_count;

        Ok(true)
    }
}
