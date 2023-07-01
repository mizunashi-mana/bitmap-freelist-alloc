use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::result::Result;

use crate::internal::layout::block;
use crate::internal::layout::constants::{ALIGNMENT_SIZE, BYTE_BIT_SIZE};
use crate::internal::layout::segment;
use crate::internal::layout::subheap::SubHeap;
use crate::sys::ptr::AnyMutPtr;
use crate::sys::SysMemEnv;
use crate::util;

pub struct Config {
    pub segment_size: usize,
    pub min_heap_size: usize,
    pub max_heap_size: usize,
}

pub struct Arena {
    context_space: AnyMutPtr,
}

impl fmt::Debug for Arena {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Arena")
            .field("context_space", &self.context_space)
            .field("header", unsafe { &self.header() })
            .finish()
    }
}

#[derive(Debug)]
pub struct Header {
    // immutable
    page_size: usize,
    segment_size: usize,
    subheap_count: usize,
    bitmap_space_begin: AnyMutPtr,
    segment_space_begin: AnyMutPtr,
    segment_space_end: AnyMutPtr,

    // late init
    min_segment_count: usize,

    // mutable
    available_size: usize,
    committed_segment_count: usize,
    committed_bitmap_item_count: usize,
    free_segments_list: *mut segment::Header,
}

const ARENA_HEADER_SIZE: usize = size_of::<Header>();
const ARENA_SUBHEAP_SIZE: usize = size_of::<SubHeap>();
const ARENA_SUBHEAP_SPACE_OFFSET: usize = ARENA_HEADER_SIZE;
const MAX_SUBHEAP_COUNT: usize = 12;

type BitMapItem = usize;

const BITMAP_ITEM_SIZE: usize = size_of::<BitMapItem>();
const BITMAP_ITEM_BIT_SIZE: usize = BITMAP_ITEM_SIZE * BYTE_BIT_SIZE;

impl Arena {
    pub unsafe fn init<Env: SysMemEnv>(
        env: &mut Env,
        config: Config,
    ) -> Result<Self, Box<dyn Error>> {
        init_arena(env, config)
    }

    pub unsafe fn header(&self) -> &mut Header {
        match self.context_space.to_raw::<Header>().as_mut() {
            Some(header) => header,
            None => {
                panic!("unreachable: The header pointer of arena is not null after initialized.")
            }
        }
    }

    pub unsafe fn subheap(&self, index: usize) -> &mut SubHeap {
        let subheap_offset = ARENA_SUBHEAP_SPACE_OFFSET + ARENA_SUBHEAP_SIZE * index;
        match self
            .context_space
            .add(subheap_offset)
            .to_raw::<SubHeap>()
            .as_mut()
        {
            Some(subheap) => subheap,
            None => {
                panic!("unreachable: The sub-heap pointer of arena is not null after initialized.")
            }
        }
    }

    pub unsafe fn max_fixed_block_size(&self) -> usize {
        ALIGNMENT_SIZE << self.header().subheap_count
    }

    pub unsafe fn is_fixed_size_block(&self, p: AnyMutPtr) -> bool {
        let header = self.header();

        p.offset_bytes_from(header.segment_space_begin) >= 0
            && header.segment_space_end.offset_bytes_from(p) > 0
    }

    pub unsafe fn alloc_segment<Env: SysMemEnv>(
        &self,
        env: &mut Env,
    ) -> Result<bool, Box<dyn Error>> {
        alloc_segment_by_header(self.header(), env)
    }

    pub unsafe fn alloc_block_free_size<Env: SysMemEnv>(
        &self,
        env: &mut Env,
        block_size: usize,
    ) -> Result<Option<AnyMutPtr>, Box<dyn Error>> {
        alloc_block_free_size_by_header(self.header(), env, block_size)
    }

    pub unsafe fn free_block_free_size<Env: SysMemEnv>(
        &self,
        env: &mut Env,
        p: AnyMutPtr,
    ) -> Result<(), Box<dyn Error>> {
        free_block_free_size_with_header(self.header(), env, p)
    }
}

unsafe fn init_arena<Env: SysMemEnv>(
    env: &mut Env,
    config: Config,
) -> Result<Arena, Box<dyn Error>> {
    let page_size = env.get_pagesize()?;
    let segment_size = config.segment_size;

    assert!(ALIGNMENT_SIZE >= 8);
    assert!(segment_size >= ALIGNMENT_SIZE * 4);
    assert!(util::bits::is_aligned(segment_size, ALIGNMENT_SIZE));
    assert!(util::bits::is_aligned(segment_size, page_size));
    assert!(util::bits::is_power_of_2(segment_size));
    assert!(util::bits::is_aligned(page_size, BITMAP_ITEM_SIZE));
    assert!(config.min_heap_size < config.max_heap_size);
    assert!(config.max_heap_size - config.min_heap_size >= segment_size + page_size);

    let subheap_count = calc_subheap_count(segment_size, page_size);
    let subheap_space_size = ARENA_SUBHEAP_SIZE * subheap_count;

    let header_subheap_space_size =
        util::bits::min_aligned_size(ARENA_HEADER_SIZE + subheap_space_size, BITMAP_ITEM_SIZE);
    let committed_context_space_size =
        util::bits::min_aligned_size(header_subheap_space_size, page_size);
    assert!(config.max_heap_size > committed_context_space_size);

    let init_available_size = config.max_heap_size - committed_context_space_size;
    let reserved_size_for_segments =
        util::bits::max_aligned_size(init_available_size, segment_size);
    let max_segment_count = reserved_size_for_segments / segment_size;

    let bitmap_space_size = max_segment_count * BITMAP_ITEM_SIZE;

    let context_space_size =
        util::bits::min_aligned_size(header_subheap_space_size + bitmap_space_size, page_size);
    let context_space = env.reserve(context_space_size)?;

    env.commit(context_space, committed_context_space_size)?;
    let committed_bitmap_item_count =
        (committed_context_space_size - header_subheap_space_size) / BITMAP_ITEM_SIZE;

    let segment_space_size = max_segment_count * segment_size;
    let segment_space = reserve_aligned_space(env, segment_space_size, segment_size)?;

    let bitmap_space_begin = context_space.add(header_subheap_space_size);
    let segment_space_begin = segment_space;
    let segment_space_end = segment_space_begin.add(segment_space_size);
    *context_space.to_raw() = Header {
        // immutable
        page_size,
        segment_size,
        subheap_count,
        bitmap_space_begin,
        segment_space_begin,
        segment_space_end,

        // not fixed yet
        min_segment_count: 0,

        // mutable
        available_size: init_available_size,
        committed_segment_count: 0,
        committed_bitmap_item_count,
        free_segments_list: std::ptr::null_mut(),
    };

    let subheap_space = context_space.add(ARENA_SUBHEAP_SPACE_OFFSET);
    for i in 0..subheap_count {
        SubHeap::init(subheap_space.add(ARENA_SUBHEAP_SIZE * i).to_raw());
    }

    let arena = Arena { context_space };
    let header = arena.header();

    while {
        let allocated_size = header_subheap_space_size
            + header.committed_bitmap_item_count * BITMAP_ITEM_SIZE
            + header.committed_segment_count * segment_size;
        allocated_size < config.min_heap_size
    } {
        if header.committed_bitmap_item_count == header.committed_segment_count {
            let bitmap_item_alloc_result = alloc_bitmap_items_by_header(header, env, false)?;
            assert!(bitmap_item_alloc_result);
        }

        let allocated = alloc_segment_by_header(header, env)?;
        assert!(allocated);
    }
    header.min_segment_count = header.committed_segment_count;

    Ok(arena)
}

unsafe fn alloc_segment_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
) -> Result<bool, Box<dyn Error>> {
    if header.committed_bitmap_item_count == header.committed_segment_count {
        let bitmap_item_alloc_result = alloc_bitmap_items_by_header(header, env, true)?;
        if !bitmap_item_alloc_result {
            return Ok(false);
        }
    }

    if header.available_size < header.segment_size {
        return Ok(false);
    }

    let new_segment_space_begin = header
        .segment_space_begin
        .add(header.committed_segment_count * header.segment_size);
    env.commit(new_segment_space_begin, header.segment_size)?;
    header.available_size -= header.segment_size;

    let new_segment_header: *mut segment::Header = new_segment_space_begin.to_raw();
    *new_segment_header = segment::Header {
        next: header.free_segments_list,
    };

    header.committed_segment_count += 1;
    header.free_segments_list = new_segment_header;

    Ok(true)
}

#[inline]
unsafe fn alloc_bitmap_items_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    use_force_commit: bool,
) -> Result<bool, Box<dyn Error>> {
    if header.available_size < header.page_size {
        return Ok(false);
    }

    let new_bitmap_space_begin = header
        .bitmap_space_begin
        .add(header.committed_bitmap_item_count * BITMAP_ITEM_SIZE);
    let new_bitmap_item_count = header.page_size / BITMAP_ITEM_SIZE;
    if use_force_commit {
        env.force_commit(new_bitmap_space_begin, header.page_size)?;
    } else {
        env.commit(new_bitmap_space_begin, header.page_size)?;
    }
    header.available_size -= header.page_size;

    std::ptr::write_bytes(new_bitmap_space_begin.to_raw::<u8>(), 0, header.page_size);

    header.committed_bitmap_item_count += new_bitmap_item_count;

    Ok(true)
}

const BLOCK_FREE_SIZE_HEADER_SIZE: usize = size_of::<block::HeaderForFreeSize>();
unsafe fn alloc_block_free_size_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    block_size: usize,
) -> Result<Option<AnyMutPtr>, Box<dyn Error>> {
    let allocate_size = BLOCK_FREE_SIZE_HEADER_SIZE + block_size;

    if header.available_size < allocate_size {
        return Ok(None);
    }

    let block_ptr = env.alloc(allocate_size)?;
    header.available_size -= allocate_size;
    block::HeaderForFreeSize::init(block_ptr.to_raw(), block_size);

    Ok(Some(block_ptr.add(BLOCK_FREE_SIZE_HEADER_SIZE)))
}

unsafe fn free_block_free_size_with_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    p: AnyMutPtr,
) -> Result<(), Box<dyn Error>> {
    let block_ptr = p.sub(BLOCK_FREE_SIZE_HEADER_SIZE);
    let block_header: *mut block::HeaderForFreeSize = block_ptr.to_raw();
    let block_size = (*block_header).block_size();
    let block_whole_size = BLOCK_FREE_SIZE_HEADER_SIZE + block_size;

    env.release(block_ptr, block_whole_size)?;
    header.available_size += block_whole_size;

    Ok(())
}

fn calc_subheap_count(segment_size: usize, page_size: usize) -> usize {
    let mut max_block_size = ALIGNMENT_SIZE;
    for subheap_count in 1..MAX_SUBHEAP_COUNT {
        if {
            max_block_size >= page_size
                || max_block_size * (BITMAP_ITEM_BIT_SIZE / 2) >= segment_size
        } {
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
        env.release(attempt_ptr.add(space_size), alignment_size)?;
        Ok(attempt_ptr)
    } else {
        let pre_adjust_size = alignment_size - post_adjust_size;
        let ptr = attempt_ptr.add(pre_adjust_size);
        env.release(attempt_ptr, pre_adjust_size)?;
        env.release(ptr.add(space_size), post_adjust_size)?;
        Ok(ptr)
    }
}
