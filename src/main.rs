use std::env;

use ::kaam::help;
use kaam::Kaam;

fn main() {
    let kaam = Kaam::new().expect("Failed to initialize kaam");
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let command = &args[1];
        match &command[..] {
            "list" => kaam.list(),
            "add" => kaam.add_kaam(&args[2..]),
            "rm" => kaam.remove(&args[2..]),
            "done" => kaam.done(&args[2..]),
            "raw" => kaam.raw(&args[2..]),
            "edit" => kaam.edit(&args[2..]),
            "sort" => kaam.sort(),
            "reset" => kaam.reset(),
            "restore" => kaam.restore(),
            "help" | "--help" | "-h" | _ => help(),
        }
    }
}
