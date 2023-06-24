pub struct AnyMutPtr {
    raw: *mut (),
}

impl AnyMutPtr {
    pub fn new<T>(raw: *mut T) -> AnyMutPtr {
        AnyMutPtr {
            raw: raw as *mut (),
        }
    }

    pub fn to_raw<T>(&self) -> *mut T {
        self.raw as *mut T
    }
}
