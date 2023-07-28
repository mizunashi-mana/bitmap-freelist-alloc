use std::ptr::NonNull;

use crate::internal::layout::segment;
use crate::internal::layout::segment_space;

#[derive(Debug)]
pub struct KeepSegmentsList {
    should_keep_count: usize,
    begin: *mut segment::CompactHeader,
    end: *mut segment::CompactHeader,
}

impl KeepSegmentsList {
    pub fn new(should_keep_count: usize) -> Self {
        Self {
            should_keep_count,
            begin: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        }
    }

    pub fn insert_and_return_flooded(
        &mut self,
        segment_space: &mut segment_space::SegmentSpace,
        seg: &mut segment::Segment,
    ) -> Option<segment::Segment> {
        todo!()
    }

    pub unsafe fn pop(
        &mut self,
        segment_space: &mut segment_space::SegmentSpace,
    ) -> Option<NonNull<segment::CompactHeader>> {
        let begin_ptr = match NonNull::new(self.begin) {
            None => return None,
            Some(begin_ptr) => begin_ptr,
        };
        let begin_seg_header = begin_ptr.as_ref();

        self.begin = begin_seg_header.next;
        match NonNull::new(begin_seg_header.next) {
            Some(begin_next_ptr) => {
                let end_ptr = NonNull::new_unchecked(self.end);

                if begin_next_ptr <= end_ptr {
                    let mut new_begin = segment_space.segment(begin_next_ptr);
                    new_begin.set_prev(std::ptr::null_mut());
                } else {
                    todo!()
                }
            }
            None => {
                self.end = std::ptr::null_mut();
            }
        }
        self.should_keep_count += 1;

        Some(begin_ptr)
    }
}
