use crate::internal::segment;

pub struct SubHeap {
    free_segments_list: *mut segment::Header,
}

impl SubHeap {
    pub unsafe fn init(p: *mut SubHeap) {
        *p = SubHeap {
            free_segments_list: std::ptr::null_mut(),
        };
    }
}
