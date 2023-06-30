use std::error::Error;
use std::result::Result;
use std::io::Write;

mod sys;
use crate::sys::SysMemEnv;

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
    main_try().unwrap();
}

fn main_try() -> Result<(), Box<dyn Error>> {
    let mut env = sys::new_env();
    let mut buffer = String::new();
    let stdin = std::io::stdin();

    let _ = unsafe { arena::Arena::init(&mut env, ARENA_CONFIG) }?;
    let size = 1024 * 1024 * 1024;

    let p = unsafe { env.reserve(size) }?;
    print!("Reserved: ");
    std::io::stdout().flush()?;
    stdin.read_line(&mut buffer)?;

    unsafe { env.commit(&p, size) }?;
    print!("Commited: ");
    std::io::stdout().flush()?;
    stdin.read_line(&mut buffer)?;

    {
        let i = p.to_raw::<[i32; 1024]>();
        unsafe { *i = [1; 1024] };
        print!("Write and read {}: ", unsafe { (*i)[0] });
        std::io::stdout().flush()?;
        stdin.read_line(&mut buffer)?;
    }

    unsafe { env.soft_decommit(&p, size) }?;
    print!("Uncommited: ");
    std::io::stdout().flush()?;
    stdin.read_line(&mut buffer)?;

    Ok(())
}
