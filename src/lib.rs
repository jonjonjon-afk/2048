//! Pure 2048 core: board rules, deterministic RNG and high-score storage.
//!
//! The crate has no terminal or clock dependencies. The shell supplies seeds
//! and, later, animation time; everything here is deterministic and testable.

pub mod cli;
pub mod game;
pub mod highscore;
pub mod rng;

pub use game::{Board, Direction, Game, Merge, MoveOutcome, SlideOutcome, Spawn};
pub use highscore::{FileStore, HighScoreError, HighScoreStore, MemoryStore};
pub use rng::Xorshift64;
