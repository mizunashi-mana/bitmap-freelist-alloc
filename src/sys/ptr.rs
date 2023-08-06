use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnyNonNullPtr {
    raw: NonNull<u8>,
}

impl AnyNonNullPtr {
    #[inline]
    pub fn new<T>(raw: NonNull<T>) -> Self {
        Self { raw: raw.cast() }
    }

    #[inline]
    pub fn as_nonnull<T>(&self) -> NonNull<T> {
        self.raw.cast()
    }

    #[inline]
    pub unsafe fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.as_nonnull().as_ptr()
    }

    #[inline]
    pub fn as_addr(&self) -> usize {
        self.raw.as_ptr() as usize
    }

    #[inline]
    pub unsafe fn as_ref<'a, T>(&self) -> &'a T {
        self.raw.cast().as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<'a, T>(&mut self) -> &'a mut T {
        self.raw.cast().as_mut()
    }

    #[inline]
    pub unsafe fn add(&self, size_bytes: usize) -> Self {
        Self {
            raw: NonNull::new_unchecked(self.raw.as_ptr().add(size_bytes)),
        }
    }

    #[inline]
    pub unsafe fn sub(&self, size_bytes: usize) -> Self {
        Self {
            raw: NonNull::new_unchecked(self.raw.as_ptr().sub(size_bytes)),
        }
    }

    #[inline]
    pub unsafe fn offset_bytes_from(&self, another: Self) -> isize {
        self.raw.as_ptr().offset_from(another.raw.as_ptr())
    }
}
