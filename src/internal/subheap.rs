use crate::internal::segment;

pub struct SubHeap {
    free_segment: *mut segment::Header,
}

impl SubHeap {
    pub unsafe fn init(p: *mut SubHeap) {
        *p = SubHeap {
            free_segment: std::ptr::null_mut(),
        };
    }
}
