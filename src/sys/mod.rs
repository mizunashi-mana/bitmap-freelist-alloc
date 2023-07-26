use std::error::Error;
use std::result::Result;

mod linux;
pub mod ptr;

use crate::util;
use ptr::AnyNonNullPtr;

pub trait SysMemEnv {
    unsafe fn get_pagesize(&mut self) -> Result<usize, Box<dyn Error>>;
    unsafe fn reserve(&mut self, len: usize) -> Result<AnyNonNullPtr, Box<dyn Error>>;
    unsafe fn alloc(&mut self, len: usize) -> Result<AnyNonNullPtr, Box<dyn Error>>;
    unsafe fn commit(&mut self, addr: AnyNonNullPtr, len: usize) -> Result<(), Box<dyn Error>>;
    unsafe fn force_commit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn soft_decommit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn hard_decommit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn release(&mut self, addr: AnyNonNullPtr, len: usize) -> Result<(), Box<dyn Error>>;

    unsafe fn reserve_aligned_space(
        &mut self,
        space_size: usize,
        alignment_size: usize,
    ) -> Result<AnyNonNullPtr, Box<dyn Error>> {
        assert!(util::bits::is_aligned(space_size, alignment_size));

        let attempt_ptr = self.reserve(space_size + alignment_size)?;
        let attempt_ptr_raw_addr = attempt_ptr.as_addr();

        let post_adjust_size = attempt_ptr_raw_addr % alignment_size;

        if util::bits::is_aligned(attempt_ptr_raw_addr, alignment_size) {
            self.release(attempt_ptr.add(space_size), alignment_size)?;
            Ok(attempt_ptr)
        } else {
            let pre_adjust_size = alignment_size - post_adjust_size;
            let ptr = attempt_ptr.add(pre_adjust_size);
            self.release(attempt_ptr, pre_adjust_size)?;
            self.release(ptr.add(space_size), post_adjust_size)?;
            Ok(ptr)
        }
    }
}

pub type SysMemEnvImpl = SysMemEnvForLinux;

pub fn new_env() -> SysMemEnvImpl {
    SysMemEnvForLinux {
        prefer_commit_strategy: linux::CommitStrategy::MprotectRw,
        prefer_soft_decommit_strategy: linux::SoftDecommitStrategy::MadviseFree,
        prefer_hard_decommit_strategy: linux::HardDecommitStrategy::MprotectNone,
    }
}

#[derive(Debug)]
pub struct SysMemEnvForLinux {
    prefer_commit_strategy: linux::CommitStrategy,
    prefer_soft_decommit_strategy: linux::SoftDecommitStrategy,
    prefer_hard_decommit_strategy: linux::HardDecommitStrategy,
}

impl SysMemEnv for SysMemEnvForLinux {
    unsafe fn get_pagesize(&mut self) -> Result<usize, Box<dyn Error>> {
        linux::get_pagesize()
    }

    unsafe fn reserve(&mut self, len: usize) -> Result<AnyNonNullPtr, Box<dyn Error>> {
        linux::reserve(len)
    }

    unsafe fn commit(&mut self, addr: AnyNonNullPtr, len: usize) -> Result<(), Box<dyn Error>> {
        self.prefer_commit_strategy = linux::commit(addr, len, self.prefer_commit_strategy)?;
        Ok(())
    }

    unsafe fn force_commit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        linux::force_commit(addr, len)
    }

    unsafe fn soft_decommit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_soft_decommit_strategy =
            linux::soft_decommit(addr, len, self.prefer_soft_decommit_strategy)?;
        Ok(())
    }

    unsafe fn hard_decommit(
        &mut self,
        addr: AnyNonNullPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_hard_decommit_strategy =
            linux::hard_decommit(addr, len, self.prefer_hard_decommit_strategy)?;
        Ok(())
    }

    unsafe fn alloc(&mut self, len: usize) -> Result<AnyNonNullPtr, Box<dyn Error>> {
        linux::alloc(len)
    }

    unsafe fn release(&mut self, addr: AnyNonNullPtr, len: usize) -> Result<(), Box<dyn Error>> {
        linux::release(addr, len)
    }
}
