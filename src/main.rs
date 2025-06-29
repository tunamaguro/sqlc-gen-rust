use sqlc_gen_rust::{StackErrorExt as _, try_main};

fn main() {
    match try_main() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("generation failed.");
            let stack = e.stack_error().join("\n");
            eprintln!("{}", stack);
            std::process::exit(1)
        }
    }
}
