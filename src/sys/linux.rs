extern crate libc;

use std::error::Error;
use std::io;
use std::result::Result;

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

pub unsafe fn reserve(len: usize) -> Result<AnyMutPtr, Box<dyn Error>> {
    let p = libc::mmap(
        std::ptr::null_mut(),
        len,
        libc::PROT_NONE,
        libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
        -1,
        0,
    );
    if p == libc::MAP_FAILED {
        Err(Box::new(io::Error::last_os_error()))
    } else {
        Ok(AnyMutPtr::new(p))
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum CommitStrategy {
    MprotectRw,
    MmapFixedProtRw,
}

pub unsafe fn commit(
    addr: &AnyMutPtr,
    len: usize,
    prefer_strategy: CommitStrategy,
) -> Result<CommitStrategy, Box<dyn Error>> {
    if prefer_strategy <= CommitStrategy::MprotectRw {
        // mprotect was added in Linux 4.9.
        let r = libc::mprotect(addr.to_raw(), len, libc::PROT_READ | libc::PROT_WRITE);
        if r == 0 {
            return Ok(CommitStrategy::MprotectRw);
        }
    }

    // Remapping FIXED region is an unrecommended strategy.
    // Use as a fallback if we cannot use mprotect.
    let p = libc::mmap(
        addr.to_raw(),
        len,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | libc::MAP_FIXED,
        -1,
        0,
    );
    if p == libc::MAP_FAILED {
        Err(Box::new(io::Error::last_os_error()))
    } else {
        Ok(CommitStrategy::MmapFixedProtRw)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub enum UncommitStrategy {
    MadviseFree,
    MadviseDontNeed,
    MmapFixedProtNone,
}

pub unsafe fn uncommit(
    addr: &AnyMutPtr,
    len: usize,
    prefer_strategy: UncommitStrategy,
) -> Result<UncommitStrategy, Box<dyn Error>> {
    if prefer_strategy <= UncommitStrategy::MadviseFree {
        // MADV_FREE was added in Linux 4.5.
        let r = libc::madvise(addr.to_raw(), len, libc::MADV_FREE);
        if r == 0 {
            return Ok(UncommitStrategy::MadviseFree);
        }
    }

    if prefer_strategy <= UncommitStrategy::MadviseDontNeed {
        // Since Linux 3.18, support for madvise is optional.
        let r = libc::madvise(addr.to_raw(), len, libc::MADV_DONTNEED);
        if r == 0 {
            return Ok(UncommitStrategy::MadviseDontNeed);
        }
    }

    // Remapping FIXED region is an unrecommended strategy.
    // Use as a fallback if we cannot use madvise.
    let p = libc::mmap(
        addr.to_raw(),
        len,
        libc::PROT_NONE,
        libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | libc::MAP_FIXED,
        -1,
        0,
    );
    if p == libc::MAP_FAILED {
        Err(Box::new(io::Error::last_os_error()))
    } else {
        Ok(UncommitStrategy::MmapFixedProtNone)
    }
}
