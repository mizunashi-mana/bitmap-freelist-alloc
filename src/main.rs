mod sys;

fn main() {
    let p = unsafe { sys::linux::reserve(16) }.unwrap();
    unsafe { sys::linux::commit(&p, 16, sys::linux::CommitStrategy::MprotectRw) }.unwrap();
    unsafe { sys::linux::uncommit(&p, 16, sys::linux::UncommitStrategy::MadviseFree) }.unwrap();

    println!("Hello, world!");
}
