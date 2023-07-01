#[derive(Debug, Clone, Copy)]
pub struct AnyMutPtr {
    raw: *mut u8,
}

impl AnyMutPtr {
    #[inline]
    pub fn new<T>(raw: *mut T) -> AnyMutPtr {
        AnyMutPtr {
            raw: raw as *mut u8,
        }
    }

    #[inline]
    pub fn to_raw<T>(&self) -> *mut T {
        self.raw as *mut T
    }

    #[inline]
    pub fn to_raw_addr(&self) -> usize {
        self.raw as usize
    }

    #[inline]
    pub unsafe fn add(&self, size_bytes: usize) -> AnyMutPtr {
        AnyMutPtr {
            raw: self.raw.add(size_bytes),
        }
    }

    #[inline]
    pub unsafe fn sub(&self, size_bytes: usize) -> AnyMutPtr {
        AnyMutPtr {
            raw: self.raw.sub(size_bytes),
        }
    }

    #[inline]
    pub unsafe fn offset_bytes_from(&self, another: AnyMutPtr) -> isize {
        self.raw.offset_from(another.raw)
    }
}
