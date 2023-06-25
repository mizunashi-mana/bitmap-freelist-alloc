use std::error::Error;
use std::result::Result;
use std::mem::size_of;

use crate::sys;
use crate::internal::subheap;

pub struct Config {
    pub segment_size: usize,
    pub min_heap_size: usize,
    pub max_heap_size: usize,
}

pub struct Arena {
    start: *mut (),
}

const ALIGNMENT_SIZE: usize = 2 * size_of::<usize>();

const ARENA_SEGMENT_SIZE_OFFSET: usize = 0;
const ARENA_MIN_SEGMENT_COUNT_OFFSET: usize = ARENA_SEGMENT_SIZE_OFFSET + size_of::<usize>();
const ARENA_MAX_SEGMENT_COUNT_OFFSET: usize = ARENA_MIN_SEGMENT_COUNT_OFFSET + size_of::<usize>();
const ARENA_SUBHEAP_COUNT_OFFSET: usize = ARENA_MAX_SEGMENT_COUNT_OFFSET + size_of::<usize>();
const ARENA_SUBHEAPS_OFFSET: usize = ARENA_SUBHEAP_COUNT_OFFSET + size_of::<usize>();
const ARENA_SUBHEAP_ITEM_SIZE: usize = size_of::<subheap::SubHeap>();

impl Arena {
    pub unsafe fn init<Env: sys::SysMemEnv>(env: &mut Env, config: Config) -> Result<Arena, Box<dyn Error>> {
        assert!(ALIGNMENT_SIZE >= 8);
        assert!(config.segment_size >= ALIGNMENT_SIZE * 4);
        assert!(config.segment_size % ALIGNMENT_SIZE == 0);

        let p = env.reserve(16)?;
        env.commit(&p, 16)?;

        let arena = Arena {
            start: p.to_raw(),
        };

        let subheap_count = calc_subheap_count(config.segment_size);

        unsafe {
            *arena.segment_size() = config.segment_size;
            *arena.min_segment_count() = 0;
            *arena.max_segment_count() = 0;
            *arena.subheap_count() = subheap_count;

            for i in 0..subheap_count {
                subheap::SubHeap::init(arena.subheap_item(i));
            }
        }

        Ok(arena)
    }

    pub unsafe fn segment_size(&self) -> *mut usize {
        self.start.add(ARENA_SEGMENT_SIZE_OFFSET) as *mut usize
    }

    pub unsafe fn min_segment_count(&self) -> *mut usize {
        self.start.add(ARENA_MIN_SEGMENT_COUNT_OFFSET) as *mut usize
    }

    pub unsafe fn max_segment_count(&self) -> *mut usize {
        self.start.add(ARENA_MAX_SEGMENT_COUNT_OFFSET) as *mut usize
    }

    pub unsafe fn subheap_count(&self) -> *mut usize {
        self.start.add(ARENA_MAX_SEGMENT_COUNT_OFFSET) as *mut usize
    }

    pub unsafe fn subheap_item(&self, index: usize) -> *mut subheap::SubHeap {
        self.start.add(ARENA_SUBHEAPS_OFFSET + ARENA_SUBHEAP_ITEM_SIZE * index) as *mut subheap::SubHeap
    }
}

const MAX_SUBHEAP_COUNT: usize = 10;
fn calc_subheap_count(segment_size: usize) -> usize {
    let mut max_block_size = ALIGNMENT_SIZE;
    for subheap_count in 1..MAX_SUBHEAP_COUNT {
        if max_block_size * 4 >= segment_size {
            return subheap_count;
        } else {
            max_block_size = max_block_size << 1;
        }
    }

    MAX_SUBHEAP_COUNT
}
