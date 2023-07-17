use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::result::Result;

use crate::internal::layout::block;
use crate::internal::layout::constants::ALIGNMENT_SIZE;
use crate::internal::layout::segment;
use crate::internal::layout::subheap;
use crate::sys::ptr::AnyMutPtr;
use crate::sys::SysMemEnv;
use crate::util;

pub struct Config {
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
    segment_compact_header_space: AnyMutPtr,
    segment_space_begin: AnyMutPtr,
    segment_space_end: AnyMutPtr,

    // late init
    min_segment_count: usize,

    // mutable
    available_size: usize,
    committed_segment_count: usize,
    committed_segment_compact_header_count: usize,
    free_segments_list: *mut segment::CompactHeader,
    subheaps: [subheap::SubHeap; subheap::CLASS_COUNT],
}

const ARENA_HEADER_SIZE: usize = size_of::<Header>();

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

    assert!(ALIGNMENT_SIZE >= 4);
    assert!(util::bits::is_aligned(segment::SEGMENT_SIZE, page_size));
    assert!(util::bits::is_aligned(page_size, segment::COMPACT_HEADER_SIZE));
    assert!(config.min_heap_size < config.max_heap_size);
    assert!(config.max_heap_size - config.min_heap_size >= segment::SEGMENT_SIZE + page_size);

    let committed_context_space_size =
        util::bits::min_aligned_size(ARENA_HEADER_SIZE, page_size);
    assert!(config.max_heap_size > committed_context_space_size);

    let init_available_size = config.max_heap_size - committed_context_space_size;
    let reserved_size_for_segments =
        util::bits::max_aligned_size(init_available_size, segment::SEGMENT_SIZE);
    let max_segment_count = reserved_size_for_segments / segment::SEGMENT_SIZE;

    let segment_compact_header_space_size = max_segment_count * segment::COMPACT_HEADER_SIZE;

    let context_space_size = util::bits::min_aligned_size(
        ARENA_HEADER_SIZE + segment_compact_header_space_size,
        page_size,
    );
    let context_space = env.reserve(context_space_size)?;

    env.commit(context_space, committed_context_space_size)?;
    let arena_header_size_aligned = util::bits::min_aligned_size(
        ARENA_HEADER_SIZE,
        segment::COMPACT_HEADER_SIZE,
    );
    let committed_segment_compact_header_count =
        (committed_context_space_size - arena_header_size_aligned) / segment::COMPACT_HEADER_SIZE;

    let segment_space_size = max_segment_count * segment::SEGMENT_SIZE;
    let segment_space = env.reserve_aligned_space(segment_space_size, segment::SEGMENT_SIZE)?;

    let segment_compact_header_space = context_space.add(arena_header_size_aligned);
    let segment_space_begin = segment_space;
    let segment_space_end = segment_space_begin.add(segment_space_size);
    *context_space.to_raw() = Header {
        // immutable
        page_size,
        segment_compact_header_space,
        segment_space_begin,
        segment_space_end,

        // not fixed yet
        min_segment_count: 0,

        // mutable
        available_size: init_available_size,
        committed_segment_count: 0,
        committed_segment_compact_header_count,
        free_segments_list: std::ptr::null_mut(),
        subheaps: [subheap::SubHeap {
            free_segments_list: std::ptr::null_mut(),
        }; subheap::CLASS_COUNT],
    };

    let arena = Arena { context_space };
    let header = arena.header();

    while {
        let allocated_size = arena_header_size_aligned
            + header.committed_segment_compact_header_count * segment::COMPACT_HEADER_SIZE
            + header.committed_segment_count * segment::SEGMENT_SIZE;
        allocated_size < config.min_heap_size
    } {
        if header.committed_segment_compact_header_count == header.committed_segment_count {
            let alloc_result = alloc_segment_compact_header_by_header(header, env, false)?;
            assert!(alloc_result);
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
    if header.committed_segment_compact_header_count == header.committed_segment_count {
        let alloc_result = alloc_segment_compact_header_by_header(header, env, true)?;
        if !alloc_result {
            return Ok(false);
        }
    }

    if header.available_size < segment::SEGMENT_SIZE {
        return Ok(false);
    }

    let new_segment_space_begin = header
        .segment_space_begin
        .add(header.committed_segment_count * segment::SEGMENT_SIZE);
    let segment_compact_header_ptr = header
        .segment_compact_header_space
        .add(header.committed_segment_count * segment::COMPACT_HEADER_SIZE);
    env.commit(new_segment_space_begin, segment::SEGMENT_SIZE)?;
    header.available_size -= segment::SEGMENT_SIZE;

    let new_segment: *mut segment::Header = new_segment_space_begin.to_raw();
    add_segment_to_free_segments(
        header,
        new_segment,
    );

    header.committed_segment_count += 1;

    Ok(true)
}

unsafe fn add_segment_to_free_segments(
    header: &mut Header,
    segment: *mut segment::Header,
) {
    let segment_compact_header_ptr = segment_ptr_to_compact_header(header, segment);
    *segment = segment::Header {
        prev: std::ptr::null_mut(),
        subheap_index: -1,
    };
    if let Some(current_free_segment_head) = header.free_segments_list.as_mut() {
        current_free_segment_head.prev = segment;
    }
    header.free_segments_list = segment;
}

unsafe fn segment_ptr_to_compact_header(
    header: &mut Header,
    segment: *mut segment::Header,
) -> *mut segment::CompactHeader {
    let segment_index = AnyMutPtr::new(segment).offset_bytes_from(header.segment_space_begin) / segment::SEGMENT_SIZE;
    let segment_compact_header_ptr = header.segment_compact_header_space
        .add(segment_index * segment::COMPACT_HEADER_SIZE);

    segment_compact_header_ptr.to_raw()
}

#[inline]
unsafe fn alloc_segment_compact_header_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    use_force_commit: bool,
) -> Result<bool, Box<dyn Error>> {
    if header.available_size < header.page_size {
        return Ok(false);
    }

    let new_segment_compact_header_space_begin = header
        .segment_compact_header_space
        .add(header.committed_segment_compact_header_count * segment::COMPACT_HEADER_SIZE);
    let new_segment_compact_headers_count = header.page_size / segment::COMPACT_HEADER_SIZE;
    if use_force_commit {
        env.force_commit(new_segment_compact_header_space_begin, header.page_size)?;
    } else {
        env.commit(new_segment_compact_header_space_begin, header.page_size)?;
    }
    header.available_size -= header.page_size;

    std::ptr::write_bytes(
        new_segment_compact_header_space_begin.to_raw::<u8>(),
        0,
        header.page_size
    );

    header.committed_segment_compact_header_count += new_segment_compact_headers_count;

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
