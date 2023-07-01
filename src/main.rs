use std::error::Error;
use std::result::Result;

mod sys;

mod internal;
use internal::arena;

mod util;

const SEGMENT_SIZE: usize = 1 << 17;
const ARENA_CONFIG: arena::Config = arena::Config {
    segment_size: SEGMENT_SIZE,
    min_heap_size: SEGMENT_SIZE * 16,
    max_heap_size: 1 << 26,
};

fn main() {
    unsafe { main_try() }.unwrap();
}

unsafe fn main_try() -> Result<(), Box<dyn Error>> {
    let mut env = sys::new_env();

    let arena = arena::Arena::init(&mut env, ARENA_CONFIG)?;
    let Some(ptr) = arena.alloc_block_free_size(&mut env, 1 << 18)? else { panic!("unexpected.") };

    let item: *mut usize = ptr.to_raw();
    *item = 0;
    println!("{:?}", *item);

    *item = ptr.to_raw_addr();
    println!("{:?}", *item);

    *item = 10;
    println!("{:?}", *item);

    Ok(())
}
