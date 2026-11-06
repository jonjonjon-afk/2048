//! Binary entry point: parse flags, run the headless dump, or print usage.
//!
//! The TUI shell is added in M2. Nothing here sets up a terminal.

use game2048::cli::{default_seed, headless_dump, parse, USAGE};

fn main() {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("error: {error}");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    };

    if args.help || !args.dump {
        print!("{USAGE}");
        return;
    }

    let seed = args.seed.unwrap_or_else(default_seed);
    print!("{}", headless_dump(seed, &args.moves));
}
