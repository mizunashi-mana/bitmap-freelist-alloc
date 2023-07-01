use std::error::Error;
use std::result::Result;

mod linux;
pub mod ptr;

use ptr::AnyMutPtr;

pub trait SysMemEnv {
    unsafe fn get_pagesize(&mut self) -> Result<usize, Box<dyn Error>>;
    unsafe fn reserve(&mut self, len: usize) -> Result<AnyMutPtr, Box<dyn Error>>;
    unsafe fn alloc(&mut self, len: usize) -> Result<AnyMutPtr, Box<dyn Error>>;
    unsafe fn commit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>>;
    unsafe fn soft_decommit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>>;
    unsafe fn hard_decommit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>>;
    unsafe fn release(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>>;
}

pub type SysMemEnvImpl = SysMemEnvForLinux;

pub fn new_env() -> SysMemEnvImpl {
    SysMemEnvForLinux {
        prefer_commit_strategy: linux::CommitStrategy::MprotectRw,
        prefer_soft_decommit_strategy: linux::SoftDecommitStrategy::MadviseFree,
        prefer_hard_decommit_strategy: linux::HardDecommitStrategy::MprotectNone,
    }
}

pub struct SysMemEnvForLinux {
    prefer_commit_strategy: linux::CommitStrategy,
    prefer_soft_decommit_strategy: linux::SoftDecommitStrategy,
    prefer_hard_decommit_strategy: linux::HardDecommitStrategy,
}

impl SysMemEnv for SysMemEnvForLinux {
    unsafe fn get_pagesize(&mut self) -> Result<usize, Box<dyn Error>> {
        linux::get_pagesize()
    }

    unsafe fn reserve(&mut self, len: usize) -> Result<AnyMutPtr, Box<dyn Error>> {
        linux::reserve(len)
    }

    unsafe fn commit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
        self.prefer_commit_strategy = linux::commit(addr, len, self.prefer_commit_strategy)?;
        Ok(())
    }

    unsafe fn soft_decommit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
        self.prefer_soft_decommit_strategy =
            linux::soft_decommit(addr, len, self.prefer_soft_decommit_strategy)?;
        Ok(())
    }

    unsafe fn hard_decommit(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
        self.prefer_hard_decommit_strategy =
            linux::hard_decommit(addr, len, self.prefer_hard_decommit_strategy)?;
        Ok(())
    }

    unsafe fn alloc(&mut self, len: usize) -> Result<AnyMutPtr, Box<dyn Error>> {
        linux::alloc(len)
    }

    unsafe fn release(&mut self, addr: AnyMutPtr, len: usize) -> Result<(), Box<dyn Error>> {
        linux::release(addr, len)
    }
}
