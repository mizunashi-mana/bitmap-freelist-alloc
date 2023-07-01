use std::error::Error;
use std::result::Result;

use crate::internal::layout::arena;
use crate::sys::ptr::AnyMutPtr;
use crate::sys::SysMemEnv;

#[derive(Debug)]
pub struct SampleAlloc {
    arena: arena::Arena,
}

impl SampleAlloc {
    pub unsafe fn init<Env: SysMemEnv>(
        env: &mut Env,
        arena_config: arena::Config,
    ) -> Result<Self, Box<dyn Error>> {
        let arena = arena::Arena::init(
            env,
            arena_config,
        )?;

        Ok(Self {
            arena,
        })
    }

    fn heap_overflow(&mut self) -> Box<dyn Error> {
        Box::new(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "Over the max heap size."))
    }

    pub unsafe fn alloc_with_env<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        size: usize,
    ) -> Result<AnyMutPtr, Box<dyn Error>> {
        if size <= self.arena.max_fixed_block_size() {
            todo!()
        } else {
            match self.arena.alloc_block_free_size(env, size)? {
                Some(block_ptr) => {
                    Ok(block_ptr)
                }
                None => {
                    Err(self.heap_overflow())
                }
            }
        }
    }

    pub unsafe fn free_with_env<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        p: AnyMutPtr,
    ) -> Result<(), Box<dyn Error>> {
        if self.arena.is_fixed_size_block(p) {
            todo!()
        } else {
            self.arena.free_block_free_size(env, p)
        }
    }
}
