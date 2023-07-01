extern crate libc;

use std::error::Error;
use std::io;
use std::result::Result;

use crate::sys::ptr::AnyMutPtr;

pub unsafe fn get_pagesize() -> Result<usize, Box<dyn Error>> {
    let v = libc::sysconf(libc::_SC_PAGE_SIZE);
    if v < 0 {
        Err(Box::new(io::Error::last_os_error()))
    } else {
        Ok(v as usize)
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

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CommitStrategy {
    MprotectRw,
    MmapFixedProtRw,
}

pub unsafe fn commit(
    addr: AnyMutPtr,
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

pub unsafe fn force_commit(addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
    // Remapping FIXED region is an unrecommended strategy.
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
        Ok(())
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub enum SoftDecommitStrategy {
    MadviseFree,
    MadviseDontNeed,
    MmapFixedRemap,
}

pub unsafe fn soft_decommit(
    addr: AnyMutPtr,
    len: usize,
    prefer_strategy: SoftDecommitStrategy,
) -> Result<SoftDecommitStrategy, Box<dyn Error>> {
    if prefer_strategy <= SoftDecommitStrategy::MadviseFree {
        // MADV_FREE was added in Linux 4.5.
        let r = libc::madvise(addr.to_raw(), len, libc::MADV_FREE);
        if r == 0 {
            return Ok(SoftDecommitStrategy::MadviseFree);
        }
    }

    if prefer_strategy <= SoftDecommitStrategy::MadviseDontNeed {
        // Since Linux 3.18, support for madvise is optional.
        let r = libc::madvise(addr.to_raw(), len, libc::MADV_DONTNEED);
        if r == 0 {
            return Ok(SoftDecommitStrategy::MadviseDontNeed);
        }
    }

    // Remapping FIXED region is an unrecommended strategy.
    // Use as a fallback if we cannot use madvise.
    // Remapping unmaps old mappings.
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
        Ok(SoftDecommitStrategy::MmapFixedRemap)
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
pub enum HardDecommitStrategy {
    MprotectNone,
    MmapFixedProtNone,
}

pub unsafe fn hard_decommit(
    addr: AnyMutPtr,
    len: usize,
    prefer_strategy: HardDecommitStrategy,
) -> Result<HardDecommitStrategy, Box<dyn Error>> {
    if prefer_strategy <= HardDecommitStrategy::MprotectNone {
        // mprotect was added in Linux 4.9.
        let r = libc::mprotect(addr.to_raw(), len, libc::PROT_NONE);
        if r == 0 {
            return Ok(HardDecommitStrategy::MprotectNone);
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
        Ok(HardDecommitStrategy::MmapFixedProtNone)
    }
}

pub unsafe fn alloc(len: usize) -> Result<AnyMutPtr, Box<dyn Error>> {
    let p = libc::mmap(
        std::ptr::null_mut(),
        len,
        libc::PROT_READ | libc::PROT_WRITE,
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

pub unsafe fn release(addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
    let p = libc::munmap(addr.to_raw(), len);
    if p != 0 {
        Err(Box::new(io::Error::last_os_error()))
    } else {
        Ok(())
    }
}
