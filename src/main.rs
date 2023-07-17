use std::error::Error;
use std::result::Result;

mod allocator;
mod internal;
mod sys;
mod util;

use crate::allocator::Allocator;

const ALLOC_CONFIG: allocator::Config = allocator::Config {
    min_heap_size: 1 << 18,
    max_heap_size: 500 << 20,
};

fn main() {
    unsafe { main_try() }.unwrap();
}

unsafe fn main_try() -> Result<(), Box<dyn Error>> {
    let mut manager = allocator::init(sys::new_env(), ALLOC_CONFIG)?;
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
