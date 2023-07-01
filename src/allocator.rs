use std::error::Error;
use std::result::Result;

use crate::internal;
use crate::sys::ptr::AnyMutPtr;
use crate::sys::SysMemEnv;

pub trait Allocator {
    unsafe fn alloc(&mut self, size: usize) -> Result<AnyMutPtr, Box<dyn Error>>;
    unsafe fn free(&mut self, p: AnyMutPtr) -> Result<(), Box<dyn Error>>;
}

pub struct Config {
    pub segment_size: usize,
    pub min_heap_size: usize,
    pub max_heap_size: usize,
}

pub unsafe fn init<Env: SysMemEnv>(
    env: Env,
    config: Config,
) -> Result<SampleAllocWithEnv<Env>, Box<dyn Error>> {
    SampleAllocWithEnv::<Env>::init(env, config)
}

pub struct SampleAllocWithEnv<Env> {
    env: Env,
    internal: internal::allocator::SampleAlloc,
}

impl<Env> SampleAllocWithEnv<Env>
    where Env: SysMemEnv
{
    unsafe fn init(mut env: Env, config: Config) -> Result<Self, Box<dyn Error>> {
        let internal = internal::allocator::SampleAlloc::init(
            &mut env,
            internal::layout::arena::Config {
                segment_size: config.segment_size,
                min_heap_size: config.min_heap_size,
                max_heap_size: config.max_heap_size,
            },
        )?;

        Ok(SampleAllocWithEnv {
            env,
            internal,
        })
    }
}

impl<Env> Allocator for SampleAllocWithEnv<Env>
    where Env: SysMemEnv
{
    unsafe fn alloc(&mut self, size: usize) -> Result<AnyMutPtr, Box<dyn Error>> {
        self.internal.alloc_with_env(&mut self.env, size)
    }

    unsafe fn free(&mut self, p: AnyMutPtr) -> Result<(), Box<dyn Error>> {
        self.internal.free_with_env(&mut self.env, p)
    }
}
