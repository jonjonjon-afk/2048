//! The pinned `--demo` / `--demo-merge` capture state.
//!
//! Seed 42 starts with a 2 at cell 0 and a 2 at cell 12. Moving Up merges them
//! at cell 0 into 4, and the post-move spawn lands at cell 9 with value 2. That
//! single fixed move is the capture state used by `make demo` and the effect
//! snapshots.

use core::time::Duration;

use crate::action::Action;
use crate::cli::DEMO_SEED;
use crate::game::{Board, Direction, Game};
use crate::update::App;

/// Duration of one animation frame in the `--frame N` contract.
pub const FRAME_MILLIS: u64 = 16;

/// The synthetic `--demo-slide` board (SPEC 8, M13).
pub const DEMO_SLIDE_BOARD: [[u32; 4]; 4] =
    [[0, 0, 2, 0], [0, 0, 2, 0], [4, 0, 0, 0], [8, 2, 0, 0]];

/// The exact elapsed time for animation frame `frame` (`N * 16 ms`).
pub fn frame_duration(frame: u64) -> Duration {
    Duration::from_millis(frame.saturating_mul(FRAME_MILLIS))
}

/// The pinned demo-merge app advanced to animation frame `frame`.
///
/// The move is the update that starts the effects, so the effect clock reads
/// exactly `frame * 16 ms` afterwards.
pub fn demo_merge_app(frame: u64) -> App {
    let mut app = App::new(Game::new(DEMO_SEED, 0)).without_slide();
    app.update(Some(Action::Move(Direction::Up)), frame_duration(frame));
    app
}

/// The animated live demo used by `make demo` and the `--demo`/`--demo-merge` TUI.
///
/// Same seed and merge move as [`demo_merge_app`], but slides stay enabled, so
/// the live game shows the 140 ms glide and the 240 ms merge jiggle. Only the
/// pinned `--render-once` captures keep the slide-less app.
pub fn demo_live_app() -> App {
    let mut app = App::new(Game::new(DEMO_SEED, 0));
    app.update(Some(Action::Move(Direction::Up)), Duration::ZERO);
    app
}

/// The `--demo-slide` app advanced to animation frame `frame`.
///
/// The synthetic board gets one `Down` move at t = 0 with the normal demo seed
/// (SPEC 5.6); the only random draw is the post-move spawn. The slide is then
/// driven in 16 ms steps so frames 9 and later are measured from the landing,
/// not from a single long step that would swallow the jiggle's start.
pub fn demo_slide_app(frame: u64) -> App {
    let board = Board::from_rows(DEMO_SLIDE_BOARD);
    let mut app = App::new(Game::from_state(board, 0, 0, DEMO_SEED));
    app.update(Some(Action::Move(Direction::Down)), Duration::ZERO);
    for _ in 0..frame {
        app.update(None, Duration::from_millis(FRAME_MILLIS));
    }
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

    #[test]
    fn demo_slide_timeline_is_pinned() {
        let at_three = demo_slide_app(3);
        assert!(at_three.sliding(), "frame 3 is inside the 140 ms slide");
        assert_eq!(at_three.effects.elapsed(), Duration::from_millis(48));

        let at_nine = demo_slide_app(9);
        assert!(!at_nine.sliding(), "frame 9 (144 ms) is past the landing");
        assert_eq!(at_nine.effects.elapsed(), Duration::from_millis(144));

        let settled = at_three.game.board;
        assert_eq!(settled.cell(8), 4, "the merged 4");
        assert_eq!(settled.cell(12), 8);
        assert_eq!(settled.cell(13), 2);
        assert_eq!(settled.cell(14), 4);
        assert_eq!(at_three.game.score, 4);
        // F1: no spawn cell is pinned; the derived spawn sits on its own board
        // cell with its own value.
        let spawn = at_three.game.last_spawn.expect("the demo move spawns");
        assert_eq!(settled.cell(spawn.cell), spawn.value);
        // F6: hand-derived from SPEC 5.2 with seed 42 (draw1 % 12 = 0 ->
        // empties[0] = cell 0, draw2 % 10 = 8 -> value 2).
        assert_eq!(
            at_three.game.last_spawn,
            Some(Spawn { cell: 0, value: 2 }),
            "seed-42 spawn, derived in R6"
        );
    }

    #[test]
    fn the_live_demo_app_animates_from_frame_zero() {
        let live = demo_live_app();
        assert!(live.sliding(), "the live demo starts its slide");
        assert_eq!(
            live.slide().map(|slide| slide.age),
            Some(Duration::ZERO),
            "the live demo slide starts at age 0"
        );
        assert!(
            !demo_merge_app(0).sliding(),
            "the pinned --render-once capture stays slide-less"
        );
    }

    #[test]
    fn the_demo_captures_do_not_animate() {
        for frame in [0u64, 4, 14, 24, 30] {
            let app = demo_merge_app(frame);
            assert!(!app.sliding(), "frame {frame} must not start a slide");
            assert_eq!(
                app.effects.elapsed(),
                frame_duration(frame),
                "frame {frame} keeps its single-step effect clock"
            );
        }
        let app = demo_merge_app(0);
        assert_eq!(app.effects.ripples().len(), 1, "the merge still fires");
    }
}
