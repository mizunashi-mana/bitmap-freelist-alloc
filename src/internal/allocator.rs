use std::error::Error;
use std::result::Result;

use crate::internal::layout::arena;
use crate::internal::layout::block;
use crate::internal::layout::segment;
use crate::internal::layout::subheap;
use crate::sys::ptr::AnyNonNullPtr;
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
        let arena = arena::Arena::init(env, arena_config)?;
        Ok(Self { arena })
    }

    fn heap_overflow(&mut self) -> Box<dyn Error> {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::OutOfMemory,
            "Over the max heap size.",
        ))
    }

    pub unsafe fn alloc_with_env<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        size: usize,
    ) -> Result<AnyNonNullPtr, Box<dyn Error>> {
        match subheap::class_of_size(size) {
            None => match self.arena.alloc_block_of_free_size(env, size)? {
                Some(block_ptr) => Ok(block_ptr),
                None => Err(self.heap_overflow())?,
            },
            Some(cls) => alloc_on_subheap_with_env(self, env, cls),
        }
    }

    pub unsafe fn free_with_env<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        ptr: AnyNonNullPtr,
    ) -> Result<(), Box<dyn Error>> {
        match self.arena.block_type(ptr) {
            block::Type::FreeSize => self.arena.free_block_of_free_size(env, ptr),
            block::Type::OnSubHeap => {
                let (seg, block_index) = segment::Segment::from_block_ptr(ptr);
                let cls = seg.subheap_class();
                if seg.free_block_and_check_empty(block_index) {
                    self.arena.remove_segment_from_subheap(cls, seg);
                    self.arena.insert_free_segment(env, seg)?;
                } else if seg.is_floated() {
                    self.arena.insert_free_segment_to_subheap(cls, seg);
                }
                Ok(())
            }
        }
    }
}

unsafe fn alloc_on_subheap_with_env<Env: SysMemEnv>(
    manager: &mut SampleAlloc,
    env: &mut Env,
    class_of_size: usize,
) -> Result<AnyNonNullPtr, Box<dyn Error>> {
    let (mut seg, block_index) = match manager.arena.subheap(class_of_size).next_free_segment() {
        Some(next_seg_ptr) => {
            let mut seg = manager.arena.segment(next_seg_ptr);
            let block_index = match seg.find_free_block() {
                Some(index) => index,
                None => panic!("unreachable: subheap free segments have free blocks."),
            };
            (seg, block_index)
        }
        None => {
            let block_size = subheap::SUBHEAP_SIZE_OF_CLASS[class_of_size];
            match manager.arena.pop_free_segment() {
                Some(mut free_seg) => {
                    segment::Segment::init_single_committed(&mut free_seg, block_size);
                    manager
                        .arena
                        .insert_free_segment_to_subheap(class_of_size, &mut free_seg);
                    (free_seg, 0)
                }
                None => {
                    let mut free_seg = match manager.arena.alloc_new_segment(env)? {
                        Some(free_seg) => free_seg,
                        None => Err(manager.heap_overflow())?,
                    };
                    segment::Segment::init_single_committed(&mut free_seg, block_size);
                    manager
                        .arena
                        .insert_free_segment_to_subheap(class_of_size, &mut free_seg);
                    (free_seg, 0)
                }
            }
        }
    };
    if seg.mark_block_and_check_full(block_index) {
        manager.arena.remove_segment_from_subheap(class_of_size, &mut seg);
    }
    Ok(seg.block_ptr(block_index))
}
