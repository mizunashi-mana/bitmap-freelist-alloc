use std::error::Error;
use std::result::Result;

mod linux;

pub trait SysMemEnv {
    type AnyMutPtr;

    unsafe fn reserve(
        &mut self,
        len: usize,
    ) -> Result<Self::AnyMutPtr, Box<dyn Error>>;
    unsafe fn commit(
        &mut self,
        addr: &Self::AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
    unsafe fn uncommit(
        &mut self,
        addr: &Self::AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn new_env() -> Box<dyn SysMemEnv<AnyMutPtr = linux::AnyMutPtr>> {
    Box::new(SysMemEnvForLinux {
        prefer_commit_strategy: linux::CommitStrategy::MprotectRw,
        prefer_uncommit_strategy: linux::UncommitStrategy::MadviseFree,
    })
}

struct SysMemEnvForLinux {
    prefer_commit_strategy: linux::CommitStrategy,
    prefer_uncommit_strategy: linux::UncommitStrategy,
}

impl SysMemEnv for SysMemEnvForLinux {
    type AnyMutPtr = linux::AnyMutPtr;

    unsafe fn reserve(
        &mut self,
        len: usize,
    ) -> Result<Self::AnyMutPtr, Box<dyn Error>> {
        linux::reserve(len)
    }

    unsafe fn commit(
        &mut self,
        addr: &Self::AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_commit_strategy = linux::commit(
            addr,
            len,
            self.prefer_commit_strategy,
        )?;
        Ok(())
    }

    unsafe fn uncommit(
        &mut self,
        addr: &Self::AnyMutPtr,
        len: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.prefer_uncommit_strategy = linux::uncommit(
            addr,
            len,
            self.prefer_uncommit_strategy,
        )?;
        Ok(())
    }
}
