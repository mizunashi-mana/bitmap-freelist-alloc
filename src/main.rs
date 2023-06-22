mod sys;

fn main() {
    let mut env = sys::new_env();

    let p = unsafe { env.reserve(16) }.unwrap();
    unsafe { env.commit(&p, 16) }.unwrap();
    unsafe { env.uncommit(&p, 16) }.unwrap();

    println!("Hello, world!");
}
