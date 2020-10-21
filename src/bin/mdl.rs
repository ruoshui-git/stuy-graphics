use std::{env, process};

use graphics::mdl::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("provide a path to mdl file");
        process::exit(1);
    }

    Interpreter::new(&args[1]).run().unwrap_or_else(|e| {
        eprintln!("engine error: {}", e);
        process::exit(1);
    });
}
