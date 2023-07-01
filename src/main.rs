use std::error::Error;
use std::result::Result;

mod sys;
mod internal;
mod util;
mod allocator;

use crate::allocator::Allocator;

const SEGMENT_SIZE: usize = 1 << 17;
const ALLOC_CONFIG: allocator::Config = allocator::Config {
    segment_size: SEGMENT_SIZE,
    min_heap_size: SEGMENT_SIZE * 16,
    max_heap_size: 500 << 20,
};

fn main() {
    unsafe { main_try() }.unwrap();
}

unsafe fn main_try() -> Result<(), Box<dyn Error>> {
    let mut manager = allocator::init(
        sys::new_env(),
        ALLOC_CONFIG,
    )?;
    println!("manager info: {:?}", manager);
    let ptr = manager.alloc(1 << 18)?;

    let item: *mut usize = ptr.to_raw();
    *item = 0;
    println!("{:?}", *item);

    *item = ptr.to_raw_addr();
    println!("{:?}", *item);

    *item = 10;
    println!("{:?}", *item);

    Ok(())
}
