//! A keyboard-first 2048 game: pure rules, deterministic RNG and a TUI shell.
//!
//! The game rules, RNG and storage contracts are pure and testable. The shell
//! owns the terminal and the clock; animation time is injected as a `dt` so
//! tests can render exact frames without waiting.

pub mod action;
pub mod cli;
pub mod game;
pub mod highscore;
pub mod input;
pub mod render;
pub mod rng;
pub mod shell;
pub mod terminal;
pub mod update;

pub use action::Action;
pub use game::{Board, Direction, Game, Merge, MoveOutcome, SlideOutcome, Spawn};
pub use highscore::{FileStore, HighScoreError, HighScoreStore, MemoryStore};
pub use input::action_for_key;
pub use render::frame_text;
pub use rng::Xorshift64;
pub use update::{App, Effect};
