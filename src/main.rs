use std::error::Error;
use std::result::Result;
use std::io::Write;

mod sys;

fn main() {
    main_try().unwrap();
}

fn main_try() -> Result<(), Box<dyn Error>> {
    let mut env = sys::new_env();
    let mut buffer = String::new();
    let stdin = std::io::stdin();

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
        let i = p.to_raw::<[i64; 1024]>();
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
