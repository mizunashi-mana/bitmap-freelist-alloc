use std::error::Error;
use std::result::Result;

mod linux;
mod ptr;

pub use ptr::AnyMutPtr;

pub trait SysMemEnv {
    unsafe fn reserve(
        &mut self,
        len: usize,
    ) -> Result<AnyMutPtr, Box<dyn Error>>;
    unsafe fn commit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn soft_decommit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn hard_decommit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn map(
        &mut self,
        len: usize,
    ) -> Result<AnyMutPtr, Box<dyn Error>>;
    unsafe fn unmap(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn new_env() -> Box<dyn SysMemEnv> {
    Box::new(SysMemEnvForLinux {
        prefer_commit_strategy: linux::CommitStrategy::MprotectRw,
        prefer_soft_decommit_strategy: linux::SoftDecommitStrategy::MadviseFree,
        prefer_hard_decommit_strategy: linux::HardDecommitStrategy::MprotectNone,
    })
}

struct SysMemEnvForLinux {
    prefer_commit_strategy: linux::CommitStrategy,
    prefer_soft_decommit_strategy: linux::SoftDecommitStrategy,
    prefer_hard_decommit_strategy: linux::HardDecommitStrategy,
}

impl SysMemEnv for SysMemEnvForLinux {
    unsafe fn reserve(
        &mut self,
        len: usize,
    ) -> Result<AnyMutPtr, Box<dyn Error>> {
        linux::reserve(len)
    }

    unsafe fn commit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_commit_strategy = linux::commit(
            addr,
            len,
            self.prefer_commit_strategy,
        )?;
        Ok(())
    }

    unsafe fn soft_decommit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_soft_decommit_strategy = linux::soft_decommit(
            addr,
            len,
            self.prefer_soft_decommit_strategy,
        )?;
        Ok(())
    }

    unsafe fn hard_decommit(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_hard_decommit_strategy = linux::hard_decommit(
            addr,
            len,
            self.prefer_hard_decommit_strategy,
        )?;
        Ok(())
    }

    unsafe fn map(
        &mut self,
        len: usize,
    ) -> Result<AnyMutPtr, Box<dyn Error>> {
        linux::map(len)
    }

    unsafe fn unmap(
        &mut self,
        addr: &AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        linux::unmap(addr, len)
    }
}
