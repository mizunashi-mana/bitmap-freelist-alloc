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

    pub unsafe fn insert_and_return_flooded(
        &mut self,
        segment_space: &mut segment_space::SegmentSpace,
        floated_seg: &mut segment::Segment,
    ) -> Option<segment::Segment> {
        assert!(floated_seg.is_floated());
        insert_and_return_flooded(self, segment_space, floated_seg)
    }

    pub unsafe fn pop(
        &mut self,
        segment_space: &mut segment_space::SegmentSpace,
    ) -> Option<NonNull<segment::CompactHeader>> {
        pop(self, segment_space)
    }
}

unsafe fn insert_and_return_flooded(
    keep_segments_list: &mut KeepSegmentsList,
    segment_space: &mut segment_space::SegmentSpace,
    floated_seg: &mut segment::Segment,
) -> Option<segment::Segment> {
    let seg_ptr = floated_seg.compact_header;
    match NonNull::new(keep_segments_list.begin) {
        None => {
            if keep_segments_list.should_keep_count == 0 {
                Some(*floated_seg)
            } else {
                keep_segments_list.begin = seg_ptr.as_ptr();
                keep_segments_list.end = seg_ptr.as_ptr();
                keep_segments_list.should_keep_count -= 1;

                None
            }
        }
        Some(begin_ptr) => {
            let end_ptr = NonNull::new_unchecked(keep_segments_list.end);

            let mut begin_seg = segment_space.segment_by_cmp_header(begin_ptr);
            let mut end_seg = segment_space.segment_by_cmp_header(end_ptr);

            let middle_ptr = NonNull::new_unchecked(
                begin_ptr
                    .as_ptr()
                    .add((end_ptr.as_ptr().offset_from(begin_ptr.as_ptr()) as usize) / 2),
            );

            if seg_ptr < begin_ptr {
                if begin_ptr == end_ptr && keep_segments_list.should_keep_count == 0 {
                    keep_segments_list.begin = seg_ptr.as_ptr();
                    keep_segments_list.end = seg_ptr.as_ptr();

                    Some(begin_seg)
                } else {
                    floated_seg.append(&mut begin_seg);
                    keep_segments_list.begin = seg_ptr.as_ptr();

                    if keep_segments_list.should_keep_count == 0 {
                        Some(force_pop_end_without_updating_count(
                            keep_segments_list,
                            segment_space,
                        ))
                    } else {
                        keep_segments_list.should_keep_count -= 1;
                        None
                    }
                }
            } else if end_ptr < seg_ptr {
                if keep_segments_list.should_keep_count == 0 {
                    Some(*floated_seg)
                } else {
                    end_seg.append(floated_seg);
                    keep_segments_list.end = seg_ptr.as_ptr();

                    keep_segments_list.should_keep_count -= 1;
                    None
                }
            } else if seg_ptr < middle_ptr {
                floated_seg.set_prev(begin_ptr.as_ptr());
                floated_seg.set_next(begin_seg.next());
                match NonNull::new(begin_seg.next()) {
                    Some(begin_next_ptr) => {
                        segment_space
                            .segment_by_cmp_header(begin_next_ptr)
                            .set_prev(seg_ptr.as_ptr());
                    }
                    None => {
                        keep_segments_list.end = seg_ptr.as_ptr();
                    }
                }
                begin_seg.set_next(seg_ptr.as_ptr());

                if keep_segments_list.should_keep_count == 0 {
                    Some(force_pop_end_without_updating_count(
                        keep_segments_list,
                        segment_space,
                    ))
                } else {
                    keep_segments_list.should_keep_count -= 1;
                    None
                }
            } else {
                floated_seg.set_prev(end_seg.prev());
                match NonNull::new(end_seg.prev()) {
                    Some(end_prev_ptr) => {
                        segment_space
                            .segment_by_cmp_header(end_prev_ptr)
                            .set_next(seg_ptr.as_ptr());
                    }
                    None => {
                        keep_segments_list.begin = seg_ptr.as_ptr();
                    }
                }

                if keep_segments_list.should_keep_count == 0 {
                    keep_segments_list.end = seg_ptr.as_ptr();
                    end_seg.set_prev(std::ptr::null_mut());
                    Some(end_seg)
                } else {
                    floated_seg.set_next(end_ptr.as_ptr());
                    end_seg.set_prev(seg_ptr.as_ptr());
                    keep_segments_list.should_keep_count -= 1;
                    None
                }
            }
        }
    }
}

unsafe fn pop(
    keep_segments_list: &mut KeepSegmentsList,
    segment_space: &mut segment_space::SegmentSpace,
) -> Option<NonNull<segment::CompactHeader>> {
    let begin_ptr = match NonNull::new(keep_segments_list.begin) {
        None => return None,
        Some(begin_ptr) => begin_ptr,
    };
    let begin_seg_header = begin_ptr.as_ref();

    match NonNull::new(begin_seg_header.next) {
        Some(begin_next_ptr) => {
            let end_ptr = NonNull::new_unchecked(keep_segments_list.end);

            if begin_next_ptr <= end_ptr {
                let mut new_begin = segment_space.segment_by_cmp_header(begin_next_ptr);
                keep_segments_list.begin = new_begin.compact_header.as_ptr();
                new_begin.set_prev(std::ptr::null_mut());
            } else if begin_next_ptr.as_ref().next == end_ptr.as_ptr() {
                let mut new_begin = segment_space.segment_by_cmp_header(end_ptr);
                let mut new_end = segment_space.segment_by_cmp_header(begin_next_ptr);
                keep_segments_list.begin = new_begin.compact_header.as_ptr();
                keep_segments_list.end = new_end.compact_header.as_ptr();
                new_begin.set_prev(std::ptr::null_mut());
                new_begin.set_next(new_end.compact_header.as_ptr());
                new_end.set_prev(new_begin.compact_header.as_ptr());
                new_end.set_next(std::ptr::null_mut());
            } else {
                let mut new_begin = segment_space.segment_by_cmp_header(end_ptr);
                let mut new_end = segment_space.segment_by_cmp_header(begin_next_ptr);
                let new_begin_next = new_end.next();
                let new_end_prev = new_begin.prev();
                keep_segments_list.begin = new_begin.compact_header.as_ptr();
                keep_segments_list.end = new_end.compact_header.as_ptr();
                new_begin.set_prev(std::ptr::null_mut());
                new_begin.set_next(new_begin_next);
                new_end.set_prev(new_end_prev);
                new_end.set_next(std::ptr::null_mut());
            }
        }
        None => {
            keep_segments_list.begin = std::ptr::null_mut();
            keep_segments_list.end = std::ptr::null_mut();
        }
    }
    keep_segments_list.should_keep_count += 1;

    Some(begin_ptr)
}

unsafe fn force_pop_end_without_updating_count(
    keep_segments_list: &mut KeepSegmentsList,
    segment_space: &mut segment_space::SegmentSpace,
) -> segment::Segment {
    let mut current_end = segment_space.segment_by_cmp_header(NonNull::new_unchecked(keep_segments_list.end));
    keep_segments_list.end = current_end.prev();
    NonNull::new_unchecked(current_end.prev()).as_mut().next = std::ptr::null_mut();

    current_end.set_prev(std::ptr::null_mut());
    current_end.set_next(std::ptr::null_mut());
    current_end
}
