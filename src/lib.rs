//! A keyboard-first 2048 game: pure rules, deterministic RNG and a TUI shell.
//!
//! The game rules, RNG and storage contracts are pure and testable. The shell
//! owns the terminal and the clock; animation time is injected as a `dt` so
//! tests can render exact frames without waiting.

pub mod action;
pub mod anim;
pub mod cli;
pub mod demo;
pub mod effects;
pub mod game;
pub mod highscore;
pub mod input;
pub mod render;
pub mod rng;
pub mod score;
pub mod shell;
pub mod terminal;
pub mod update;

pub use action::Action;
pub use demo::demo_merge_app;
pub use effects::Effects;
pub use game::{Board, Direction, Game, Merge, MoveOutcome, SlideOutcome, Spawn, Track};
pub use highscore::{
    FileStore, HighScoreError, HighScoreStore, HighScores, MemoryStore, ScoreEntry,
};
pub use input::action_for_key;
pub use render::frame_text;
pub use rng::Xorshift64;
pub use score::{ScoreCount, SCORE_COUNT_MILLIS};
pub use update::{App, Effect};
