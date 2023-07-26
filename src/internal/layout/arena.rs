use std::array;
use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::ptr::NonNull;
use std::result::Result;

use crate::internal::layout::block;
use crate::internal::layout::constants::ALIGNMENT_SIZE;
use crate::internal::layout::segment;
use crate::internal::layout::subheap;
use crate::sys::ptr::AnyNonNullPtr;
use crate::sys::SysMemEnv;
use crate::util;

pub struct Config {
    pub min_heap_size: usize,
    pub max_heap_size: usize,
    pub keep_segments_count: usize,
}

pub struct Arena {
    context_space: AnyNonNullPtr,
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
    segment_compact_header_space: AnyNonNullPtr,
    segment_space_begin: AnyNonNullPtr,
    segment_space_end: AnyNonNullPtr,

    // mutable
    available_size: usize,
    committed_segment_compact_header_count: usize,
    committed_segment_count: usize,
    free_segments_begin: *mut segment::CompactHeader,
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

    #[inline]
    unsafe fn header(&self) -> &Header {
        self.context_space.as_ref()
    }

    #[inline]
    unsafe fn header_mut(&mut self) -> &mut Header {
        self.context_space.as_mut()
    }

    #[inline]
    pub unsafe fn subheap(&mut self, class_of_size: usize) -> &mut subheap::SubHeap {
        assert!(class_of_size < subheap::CLASS_COUNT);

        let header = self.header_mut();
        &mut header.subheaps[class_of_size]
    }

    #[inline]
    pub unsafe fn segment(&mut self, seg_ptr: NonNull<segment::CompactHeader>) -> segment::Segment {
        segment_by_header(self.header_mut(), seg_ptr)
    }

    #[inline]
    pub unsafe fn block_type(&mut self, ptr: AnyNonNullPtr) -> block::Type {
        let header = self.header();

        if {
            ptr.offset_bytes_from(header.segment_space_begin) >= 0
                && header.segment_space_end.offset_bytes_from(ptr) > 0
        } {
            block::Type::OnSubHeap
        } else {
            block::Type::FreeSize
        }
    }

    #[inline]
    pub unsafe fn insert_free_segment<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        single_seg: &segment::Segment,
    ) -> Result<(), Box<dyn Error>> {
        assert!(single_seg.prev().is_null());
        assert!(single_seg.next().is_null());

        todo!()
    }

    #[inline]
    pub unsafe fn pop_free_segment(&mut self) -> Option<segment::Segment> {
        pop_free_segment_by_header(self.header_mut())
    }

    #[inline]
    pub unsafe fn insert_free_segment_to_subheap(
        &mut self,
        class_of_size: usize,
        single_seg: &mut segment::Segment,
    ) {
        assert!(single_seg.prev().is_null());
        assert!(single_seg.next().is_null());

        insert_free_segment_to_subheap_by_header(self.header_mut(), class_of_size, single_seg)
    }

    #[inline]
    pub unsafe fn remove_segment_from_subheap(&mut self, class_of_size: usize, seg: &mut segment::Segment) {
        remove_segment_from_subheap_by_header(self.header_mut(), class_of_size, seg)
    }

    #[inline]
    pub unsafe fn alloc_block_of_free_size<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        block_size: usize,
    ) -> Result<Option<AnyNonNullPtr>, Box<dyn Error>> {
        alloc_block_free_size_by_header(self.header_mut(), env, block_size)
    }

    #[inline]
    pub unsafe fn free_block_of_free_size<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
        ptr: AnyNonNullPtr,
    ) -> Result<(), Box<dyn Error>> {
        free_block_free_size_by_header(self.header_mut(), env, ptr)
    }

    pub unsafe fn alloc_new_segment<Env: SysMemEnv>(
        &mut self,
        env: &mut Env,
    ) -> Result<Option<segment::Segment>, Box<dyn Error>> {
        alloc_new_segment_by_header(self.header_mut(), env)
    }
}

unsafe fn init_arena<Env: SysMemEnv>(
    env: &mut Env,
    config: Config,
) -> Result<Arena, Box<dyn Error>> {
    let page_size = env.get_pagesize()?;

    assert!(ALIGNMENT_SIZE >= 4);
    assert!(util::bits::is_aligned(segment::SEGMENT_SIZE, page_size));
    assert!(util::bits::is_aligned(page_size, ALIGNMENT_SIZE));
    assert!(util::bits::is_aligned(
        page_size,
        segment::COMPACT_HEADER_SIZE
    ));
    assert!(config.min_heap_size < config.max_heap_size);
    assert!(config.max_heap_size - config.min_heap_size >= segment::SEGMENT_SIZE + page_size);

    let committed_context_space_size = util::bits::min_aligned_size(ARENA_HEADER_SIZE, page_size);
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
    let mut context_space = env.reserve(context_space_size)?;

    env.commit(context_space, committed_context_space_size)?;
    let arena_header_size_aligned =
        util::bits::min_aligned_size(ARENA_HEADER_SIZE, segment::COMPACT_HEADER_SIZE);
    let committed_segment_compact_header_count =
        (committed_context_space_size - arena_header_size_aligned) / segment::COMPACT_HEADER_SIZE;

    let segment_space_size = max_segment_count * segment::SEGMENT_SIZE;
    let segment_space = env.reserve_aligned_space(segment_space_size, segment::SEGMENT_SIZE)?;

    let segment_compact_header_space = context_space.add(arena_header_size_aligned);
    let segment_space_begin = segment_space;
    let segment_space_end = segment_space_begin.add(segment_space_size);
    *context_space.as_mut() = Header {
        // immutable
        page_size,
        segment_compact_header_space,
        segment_space_begin,
        segment_space_end,

        // mutable
        available_size: init_available_size,
        committed_segment_compact_header_count,
        committed_segment_count: 0,
        free_segments_begin: std::ptr::null_mut(),
        subheaps: array::from_fn(|_| subheap::SubHeap::init()),
    };

    let _ = Arena { context_space };

    todo!()
}

unsafe fn segment_by_header(
    header: &mut Header,
    seg_ptr: NonNull<segment::CompactHeader>,
) -> segment::Segment {
    let raw_seg_ptr = AnyNonNullPtr::new(seg_ptr);
    let seg_index = (raw_seg_ptr.offset_bytes_from(header.segment_compact_header_space) as usize)
        / segment::COMPACT_HEADER_SIZE;
    assert!(seg_index < header.committed_segment_compact_header_count);

    let raw_additional_header: NonNull<segment::AdditionalHeader> = header
        .segment_space_begin
        .add(seg_index * segment::SEGMENT_SIZE)
        .as_nonnull();

    segment::Segment {
        raw_compact_header: seg_ptr,
        raw_additional_header,
    }
}

const BLOCK_FREE_SIZE_HEADER_SIZE: usize = size_of::<block::HeaderForFreeSize>();
unsafe fn alloc_block_free_size_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    block_size: usize,
) -> Result<Option<AnyNonNullPtr>, Box<dyn Error>> {
    let allocate_size =
        util::bits::min_aligned_size(BLOCK_FREE_SIZE_HEADER_SIZE + block_size, header.page_size);

    if header.available_size < allocate_size {
        return Ok(None);
    }

    let block_ptr = env.alloc(allocate_size)?;
    header.available_size -= allocate_size;
    block::HeaderForFreeSize::init(block_ptr.as_nonnull(), block_size);

    Ok(Some(block_ptr.add(BLOCK_FREE_SIZE_HEADER_SIZE)))
}

unsafe fn free_block_free_size_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
    ptr: AnyNonNullPtr,
) -> Result<(), Box<dyn Error>> {
    let block_ptr = ptr.sub(BLOCK_FREE_SIZE_HEADER_SIZE);
    let block_whole_size = {
        let block_header: &block::HeaderForFreeSize = block_ptr.as_ref();
        BLOCK_FREE_SIZE_HEADER_SIZE + block_header.block_size()
    };

    env.release(block_ptr, block_whole_size)?;
    header.available_size += block_whole_size;

    Ok(())
}

unsafe fn pop_free_segment_by_header(header: &mut Header) -> Option<segment::Segment> {
    match NonNull::new(header.free_segments_begin) {
        None => None,
        Some(free_seg_header) => {
            header.free_segments_begin = free_seg_header.as_ref().next();
            todo!()
        }
    }
}

unsafe fn alloc_new_segment_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
) -> Result<Option<segment::Segment>, Box<dyn Error>> {
    if header.committed_segment_compact_header_count == header.committed_segment_count {
        if !allow_new_segment_compact_headers_by_header(header, env)? {
            return Ok(None);
        }
    }

    if header.available_size < segment::SEGMENT_SIZE {
        return Ok(None);
    }

    let committed_segment_count = header.committed_segment_count;
    let new_segment_compact_header_space_begin = header
        .segment_compact_header_space
        .add(committed_segment_count * segment::COMPACT_HEADER_SIZE);
    let new_segment_space_begin = header
        .segment_space_begin
        .add(committed_segment_count * segment::SEGMENT_SIZE);
    env.commit(new_segment_space_begin, segment::SEGMENT_SIZE)?;
    header.available_size -= segment::SEGMENT_SIZE;

    header.committed_segment_count += 1;

    Ok(Some(segment::Segment {
        raw_compact_header: new_segment_compact_header_space_begin.as_nonnull(),
        raw_additional_header: new_segment_space_begin.as_nonnull(),
    }))
}

unsafe fn allow_new_segment_compact_headers_by_header<Env: SysMemEnv>(
    header: &mut Header,
    env: &mut Env,
) -> Result<bool, Box<dyn Error>> {
    if header.available_size < header.page_size {
        return Ok(false);
    }

    let new_segment_compact_header_space_begin = header
        .segment_compact_header_space
        .add(header.committed_segment_compact_header_count * segment::COMPACT_HEADER_SIZE);
    let new_segment_compact_headers_count = header.page_size / segment::COMPACT_HEADER_SIZE;
    env.commit(new_segment_compact_header_space_begin, header.page_size)?;
    header.available_size -= header.page_size;

    header.committed_segment_compact_header_count += new_segment_compact_headers_count;

    Ok(true)
}

unsafe fn insert_free_segment_to_subheap_by_header(
    header: &mut Header,
    class_of_size: usize,
    single_seg: &mut segment::Segment,
) {
    macro_rules! subheap_cls {
        () => {
            header.subheaps[class_of_size]
        };
    }

    match NonNull::new(subheap_cls!().free_segments_begin) {
        None => {
            let seg_ptr = single_seg.pointer_to_seg();
            subheap_cls!().free_segments_begin = seg_ptr;
            subheap_cls!().free_segments_end = seg_ptr;
        }
        Some(free_segments_begin_ptr) => {
            let seg_ptr = single_seg.pointer_to_seg();
            let mut free_segments_begin = segment_by_header(header, free_segments_begin_ptr);
            if seg_ptr < free_segments_begin_ptr.as_ptr() {
                single_seg.append(&mut free_segments_begin);
                subheap_cls!().free_segments_begin = seg_ptr;
            } else {
                let free_segments_end_ptr =
                    NonNull::new_unchecked(subheap_cls!().free_segments_end);
                let mut free_segments_end = segment_by_header(header, free_segments_end_ptr);
                free_segments_end.append(single_seg);
                subheap_cls!().free_segments_end = seg_ptr;
            }
        }
    }
}

unsafe fn remove_segment_from_subheap_by_header(
    header: &mut Header,
    class_of_size: usize,
    seg: &mut segment::Segment,
) {
    macro_rules! subheap_cls {
        () => {
            header.subheaps[class_of_size]
        };
    }

    assert!(!subheap_cls!().free_segments_begin.is_null());
    assert!(!subheap_cls!().free_segments_end.is_null());

    let seg_ptr = seg.pointer_to_seg();

    if seg_ptr == subheap_cls!().free_segments_begin {
        subheap_cls!().free_segments_begin = seg.next();
    }
    if seg_ptr == subheap_cls!().free_segments_end {
        subheap_cls!().free_segments_end = seg.prev();
    }

    match NonNull::new(seg.next()) {
        Some(seg_next_ptr) => {
            let mut seg_next = segment_by_header(header, seg_next_ptr);
            seg_next.set_prev(seg.prev());
        }
        None => {
            // do nothing
        }
    }
    match NonNull::new(seg.prev()) {
        Some(seg_prev_ptr) => {
            let mut seg_prev = segment_by_header(header, seg_prev_ptr);
            seg_prev.set_next(seg.next());
        }
        None => {
            // do nothing
        }
    }

    seg.set_prev(std::ptr::null_mut());
    seg.set_next(std::ptr::null_mut());
}
