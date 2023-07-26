use std::error::Error;
use std::result::Result;

use crate::internal;
use crate::sys::ptr::AnyNonNullPtr;
use crate::sys::SysMemEnv;

pub trait Allocator {
    unsafe fn alloc(&mut self, size: usize) -> Result<AnyNonNullPtr, Box<dyn Error>>;
    unsafe fn free(&mut self, p: AnyNonNullPtr) -> Result<(), Box<dyn Error>>;
}

pub struct Config {
    pub min_heap_size: usize,
    pub max_heap_size: usize,
}

pub unsafe fn init<Env: SysMemEnv>(
    env: Env,
    config: Config,
) -> Result<SampleAllocWithEnv<Env>, Box<dyn Error>> {
    SampleAllocWithEnv::<Env>::init(env, config)
}

#[derive(Debug)]
pub struct SampleAllocWithEnv<Env> {
    env: Env,
    internal: internal::allocator::SampleAlloc,
}

impl<Env> SampleAllocWithEnv<Env>
where
    Env: SysMemEnv,
{
    unsafe fn init(mut env: Env, config: Config) -> Result<Self, Box<dyn Error>> {
        let internal = internal::allocator::SampleAlloc::init(
            &mut env,
            internal::layout::arena::Config {
                min_heap_size: config.min_heap_size,
                max_heap_size: config.max_heap_size,
                keep_segments_count: (config.min_heap_size
                    / internal::layout::segment::SEGMENT_SIZE)
                    + 12,
            },
        )?;

        Ok(SampleAllocWithEnv { env, internal })
    }
}

impl<Env> Allocator for SampleAllocWithEnv<Env>
where
    Env: SysMemEnv,
{
    unsafe fn alloc(&mut self, size: usize) -> Result<AnyNonNullPtr, Box<dyn Error>> {
        self.internal.alloc_with_env(&mut self.env, size)
    }

    unsafe fn free(&mut self, p: AnyNonNullPtr) -> Result<(), Box<dyn Error>> {
        self.internal.free_with_env(&mut self.env, p)
    }
}
