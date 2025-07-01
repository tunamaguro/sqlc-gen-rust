use sqlc_gen_rust::{StackErrorExt as _, try_main};

trait IsCopy: Copy {
    fn copilable() {}
}

impl<T: Copy> IsCopy for T {}

fn use_ownership<T>(a: T) {}

fn main() {
    let s = format!("123");
    let a = Some(&s);

    use_ownership(a);

    let b = a;

    dbg!(a);

    match try_main() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("generation failed.");
            let stack = e.stack_error().join("\n");
            eprintln!("{stack}");
            std::process::exit(1)
        }
    }
}
