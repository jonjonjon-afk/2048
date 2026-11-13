//! The pinned `--demo` / `--demo-merge` capture state.
//!
//! Seed 42 starts with a 2 at cell 0 and a 2 at cell 12. Moving Up merges them
//! at cell 0 into 4, and the post-move spawn lands at cell 9 with value 2. That
//! single fixed move is the capture state used by `make demo` and the effect
//! snapshots.

use core::time::Duration;

use crate::action::Action;
use crate::cli::DEMO_SEED;
use crate::game::{Direction, Game};
use crate::update::App;

/// Duration of one animation frame in the `--frame N` contract.
pub const FRAME_MILLIS: u64 = 16;

/// The exact elapsed time for animation frame `frame` (`N * 16 ms`).
pub fn frame_duration(frame: u64) -> Duration {
    Duration::from_millis(frame.saturating_mul(FRAME_MILLIS))
}

/// The pinned demo-merge app advanced to animation frame `frame`.
///
/// The move is the update that starts the effects, so the effect clock reads
/// exactly `frame * 16 ms` afterwards.
pub fn demo_merge_app(frame: u64) -> App {
    let mut app = App::new(Game::new(DEMO_SEED, 0));
    app.update(Some(Action::Move(Direction::Up)), frame_duration(frame));
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Spawn;

    #[test]
    fn demo_merge_board_is_pinned() {
        let app = demo_merge_app(0);

        assert_eq!(
            app.game.board.rows(),
            [[4, 0, 0, 0], [0, 0, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]]
        );
        assert_eq!(app.game.board.cell(0), 4);
        assert_eq!(app.game.board.cell(9), 2);
        assert_eq!(app.game.score, 4);
        assert_eq!(app.game.best, 4);
        assert_eq!(app.game.last_spawn, Some(Spawn { cell: 9, value: 2 }));

        assert_eq!(app.effects.ripples().len(), 1);
        assert_eq!(app.effects.ripples()[0].cell, 0);
        assert_eq!(app.effects.ripples()[0].value, 4);
        assert_eq!(app.effects.sparks().len(), crate::effects::SPARKS_PER_MERGE);
        assert!(app.effects.is_flashing(0));
        assert!(app.effects.is_popping(9));
    }

    #[test]
    fn frame_zero_starts_at_zero_elapsed() {
        assert_eq!(frame_duration(0), Duration::ZERO);
        assert_eq!(frame_duration(14), Duration::from_millis(224));
        assert_eq!(demo_merge_app(0).effects.elapsed(), Duration::ZERO);
    }
}
