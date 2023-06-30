use std::error::Error;
use std::result::Result;
use std::mem::size_of;

use crate::sys::SysMemEnv;
use crate::internal::subheap::SubHeap;
use crate::internal::segment;
use crate::util;
use crate::sys::ptr::AnyMutPtr;

pub struct Config {
    pub segment_size: usize,
    pub min_heap_size: usize,
    pub max_heap_size: usize,
}

pub struct Arena {
    context_space: AnyMutPtr,
}

const ALIGNMENT_SIZE: usize = size_of::<usize>();

struct Header {
    available_size: usize,
    subheap_count: usize,
    min_segment_count: usize,
    committed_segment_count: usize,
    committed_bitmap_item_count: usize,
    segment_space_begin: AnyMutPtr,
    segment_space_end: AnyMutPtr,
    free_segments_list: *mut segment::Header,
}

const ARENA_HEADER_SIZE: usize = size_of::<Header>();
const ARENA_SUBHEAP_ITEM_SIZE: usize = size_of::<SubHeap>();
const ARENA_SUBHEAPS_OFFSET: usize = ARENA_HEADER_SIZE;

type BitMapItem = usize;

const BYTE_BIT_SIZE: usize = 8;
const BITMAP_ITEM_SIZE: usize = size_of::<BitMapItem>();
const BITMAP_ITEM_BIT_SIZE: usize = BITMAP_ITEM_SIZE * BYTE_BIT_SIZE;

impl Arena {
    pub unsafe fn init<Env: SysMemEnv>(env: &mut Env, config: Config) -> Result<Arena, Box<dyn Error>> {
        let page_size = env.get_pagesize()?;

        assert!(ALIGNMENT_SIZE >= 8);
        assert!(config.segment_size >= ALIGNMENT_SIZE * 4);
        assert!(util::bits::is_aligned(config.segment_size, ALIGNMENT_SIZE));
        assert!(util::bits::is_aligned(config.segment_size, page_size));
        assert!(util::bits::is_power_of_2(config.segment_size));
        assert!(util::bits::is_aligned(page_size, BITMAP_ITEM_SIZE));
        assert!(config.min_heap_size < config.max_heap_size);

        let subheap_count = calc_subheap_count(config.segment_size);
        let subheap_space_size = ARENA_SUBHEAP_ITEM_SIZE * subheap_count;

        let header_subheap_space_size = util::bits::min_aligned_size(
            ARENA_HEADER_SIZE + subheap_space_size,
            BITMAP_ITEM_SIZE,
        );
        let committed_context_space_size = util::bits::min_aligned_size(
            header_subheap_space_size,
            page_size,
        );
        assert!(config.max_heap_size > committed_context_space_size);

        let init_available_size = config.max_heap_size - committed_context_space_size;
        let reserved_size_for_segments = util::bits::max_aligned_size(
            init_available_size,
            config.segment_size,
        );
        let max_segment_count = reserved_size_for_segments / config.segment_size;

        let bitmap_space_size = max_segment_count * BITMAP_ITEM_SIZE;

        let context_space_size = util::bits::min_aligned_size(
            header_subheap_space_size + bitmap_space_size,
            page_size,
        );
        let context_space = env.reserve(context_space_size)?;

        env.commit(&context_space, committed_context_space_size)?;
        let committed_bitmap_item_count
            = (committed_context_space_size - header_subheap_space_size)
            / BITMAP_ITEM_SIZE
            ;

        let segment_space_size = max_segment_count * config.segment_size;
        let segment_space = reserve_aligned_space(
            env,
            segment_space_size,
            config.segment_size,
        )?;

        let arena = Arena {
            context_space,
        };

        let segment_space_begin = segment_space;
        let segment_space_end = segment_space_begin.add(segment_space_size);
        *arena.header() = Header {
            available_size: init_available_size,
            subheap_count: subheap_count,
            // not fixed yet
            min_segment_count: 0,
            committed_segment_count: 0,
            committed_bitmap_item_count: committed_bitmap_item_count,
            segment_space_begin,
            segment_space_end,
            free_segments_list: std::ptr::null_mut(),
        };

        for i in 0..subheap_count {
            SubHeap::init(arena.subheap_item(i));
        }

        Ok(arena)
    }

    pub unsafe fn header(&self) -> *mut Header {
        self.context_space.to_raw()
    }

    pub unsafe fn subheap_item(&self, index: usize) -> *mut SubHeap {
        self.context_space.add(ARENA_SUBHEAPS_OFFSET + ARENA_SUBHEAP_ITEM_SIZE * index).to_raw()
    }

    pub unsafe fn alloc_segment(&mut self) -> Result<*mut segment::Header, Box<dyn Error>> {
        Ok(std::ptr::null_mut())
    }

    pub unsafe fn free_segment(&mut self, p: *mut segment::Header) -> Result<(), Box<dyn Error>> {
        Ok(())
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

unsafe fn reserve_aligned_space<Env: SysMemEnv>(
    env: &mut Env,
    space_size: usize,
    alignment_size: usize,
) -> Result<AnyMutPtr, Box<dyn Error>> {
    assert!(util::bits::is_aligned(space_size, alignment_size));

    let attempt_ptr = env.reserve(space_size + alignment_size)?;
    let attempt_ptr_raw_addr = attempt_ptr.to_raw_addr();

    let post_adjust_size = attempt_ptr_raw_addr % alignment_size;

    if util::bits::is_aligned(attempt_ptr_raw_addr, alignment_size) {
        env.release(&attempt_ptr.add(space_size), alignment_size)?;
        Ok(attempt_ptr)
    } else {
        let pre_adjust_size = alignment_size - post_adjust_size;
        let ptr = attempt_ptr.add(pre_adjust_size);
        env.release(&attempt_ptr, pre_adjust_size)?;
        env.release(&ptr.add(space_size), post_adjust_size)?;
        Ok(ptr)
    }
}
