//! Binary entry point: parse flags, then pick a headless mode or the TUI.
//!
//! The live game, the pty self-test and `--render-once` all render the same
//! application state. Only the live game ever opens the real high-score file;
//! dump, render-once, self-test, demo and no-persist stay in memory.

use game2048::cli::{headless_dump, parse, resolved_seed, USAGE};
use game2048::game::Game;
use game2048::highscore::{FileStore, MemoryStore};
use game2048::render::frame_text;
use game2048::shell;
use game2048::update::App;

fn main() {
    let args = match parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("error: {error}");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    };

    if args.help {
        print!("{USAGE}");
        return;
    }

    let seed = resolved_seed(&args);

    if args.dump {
        print!("{}", headless_dump(seed, &args.moves));
        return;
    }

    if args.render_once {
        let app = App::new(Game::new(seed, 0));
        print!("{}", frame_text(&app, 100, 36));
        return;
    }

    if args.self_test_tui {
        if let Err(error) = shell::run_self_test(MemoryStore::new(0), seed) {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
        return;
    }

    let keep_in_memory = args.no_persist || args.demo || args.demo_merge;
    let outcome = if keep_in_memory {
        shell::run_live(MemoryStore::new(0), seed, args.fps)
    } else {
        match FileStore::from_default_path() {
            Ok(store) => shell::run_live(store, seed, args.fps),
            Err(error) => {
                eprintln!("warning: {error}; using an in-memory high score");
                shell::run_live(MemoryStore::new(0), seed, args.fps)
            }
        }
    };

    if let Err(error) = outcome {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
