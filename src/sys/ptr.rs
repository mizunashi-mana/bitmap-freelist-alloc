pub struct AnyMutPtr {
    raw: *mut u8,
}

impl AnyMutPtr {
    pub fn new<T>(raw: *mut T) -> AnyMutPtr {
        AnyMutPtr {
            raw: raw as *mut u8,
        }
    }

    pub fn to_raw<T>(&self) -> *mut T {
        self.raw as *mut T
    }

    pub fn to_raw_addr(&self) -> usize {
        self.raw as usize
    }

    pub unsafe fn add(&self, size_bytes: usize) -> AnyMutPtr {
        AnyMutPtr {
            raw: self.raw.add(size_bytes),
        }
    }
}
