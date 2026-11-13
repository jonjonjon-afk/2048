//! M6 score count-up: the displayed score chases the real score.
//!
//! Every test drives the count with explicit `dt` values and asserts the G14 /
//! G15 literals; none reads the wall clock or renders from a fixture.

use std::time::Duration;

use game2048::demo::demo_merge_app;
use game2048::game::{Board, Direction, Game};
use game2048::render::frame_text;
use game2048::update::App;
use game2048::Action;

/// An app whose only scoring move is a left slide merging two 8s into 16.
fn sixteen_point_app() -> App {
    let board = Board::from_rows([[8, 8, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    App::new(Game::from_state(board, 0, 0, 42))
}

fn millis(value: u64) -> Duration {
    Duration::from_millis(value)
}

#[test]
fn a_merge_during_a_count_retargets_from_the_shown_value() {
    let mut app = sixteen_point_app();
    app.update(Some(Action::Move(Direction::Left)), millis(50));
    assert_eq!(app.game.score, 16);
    assert_eq!(app.shown_score(), 4, "0 -> 16 at t = 50 ms shows 4");

    // The next move adds 8 while the count is still running at 4.
    app.game.board = Board::from_rows([[4, 4, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    app.update(Some(Action::Move(Direction::Left)), millis(100));
    assert_eq!(app.game.score, 24, "the merge raised the real score to 24");
    assert_eq!(
        app.shown_score(),
        14,
        "4 + floor(20 * (1 - (1 - 100/450)^3)) = 14"
    );

    app.update(None, millis(350));
    assert_eq!(
        app.shown_score(),
        24,
        "the retargeted count lands at 450 ms"
    );
}

#[test]
fn demo_merge_frames_show_the_counted_score() {
    // G15: `--demo-merge` score 0 -> 4 at frames 4 / 14 / 24 / 30.
    let cases = [
        (4u64, 64u64, "SCORE 1"),
        (14, 224, "SCORE 3"),
        (24, 384, "SCORE 3"),
        (30, 480, "SCORE 4"),
    ];

    for (frame, elapsed, score) in cases {
        let app = demo_merge_app(frame);
        assert_eq!(app.game.score, 4, "frame {frame} real score");
        assert_eq!(
            app.effects.elapsed(),
            millis(elapsed),
            "frame {frame} advances the effect clock by its own dt"
        );

        let text = frame_text(&app, 100, 36);
        assert!(text.contains(score), "frame {frame} must show {score:?}");
        assert!(
            text.contains("BEST 4"),
            "frame {frame} must keep BEST fully caught up"
        );
        if frame < 30 {
            assert!(
                !text.contains("SCORE 4"),
                "frame {frame} must not show the un-counted score"
            );
        }
    }
}

#[test]
fn game_over_overlay_shows_the_counted_score() {
    let mut app = sixteen_point_app();
    app.update(Some(Action::Move(Direction::Left)), millis(200));
    assert_eq!(app.game.score, 16);
    assert_eq!(app.shown_score(), 13, "0 -> 16 at t = 200 ms shows 13");

    app.game.game_over = true;
    let text = frame_text(&app, 100, 36);
    assert!(text.contains("Score 13"), "overlay must count: {text}");
    assert!(
        !text.contains("Score 16"),
        "overlay must not show real: {text}"
    );
}

#[test]
fn best_jumps_while_the_score_counts() {
    let mut app = demo_merge_app(4);
    assert_eq!(app.effects.elapsed(), millis(64));
    assert_eq!(app.game.best, 4, "BEST jumps immediately");
    assert_eq!(app.shown_score(), 1, "SCORE is still counting at 64 ms");

    let text = frame_text(&app, 100, 36);
    assert!(text.contains("BEST 4"), "frame 4 must show BEST 4");
    assert!(app.effects.is_best_pulsing(), "pulse runs at 64 ms");

    app.update(None, millis(543));
    assert_eq!(app.effects.elapsed(), millis(607));
    assert!(
        !app.effects.is_best_pulsing(),
        "the 600 ms best pulse is over at 607 ms"
    );
    assert_eq!(app.shown_score(), 4, "the count itself is done by 607 ms");
}

#[test]
fn restart_resets_the_shown_score_immediately() {
    let mut app = sixteen_point_app();
    app.update(Some(Action::Move(Direction::Left)), millis(100));
    assert_eq!(app.shown_score(), 8, "0 -> 16 at t = 100 ms shows 8");

    app.update(Some(Action::Restart), Duration::ZERO);
    assert_eq!(app.game.score, 0);
    assert_eq!(app.shown_score(), 0, "restart shows 0 on the same update");

    let text = frame_text(&app, 100, 36);
    assert!(text.contains("SCORE 0"), "the next frame must show SCORE 0");
    assert!(!text.contains("SCORE 8"), "no stale counted value survives");
}

#[test]
fn a_move_without_score_starts_no_count() {
    // The 2 slides left into the empty first column; the 4 slides too. No merge.
    let board = Board::from_rows([[0, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 4]]);
    let mut app = App::new(Game::from_state(board, 0, 0, 42));

    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert_eq!(app.game.score, 0, "the move scored nothing");
    assert_eq!(app.game.board.cell(0), 2, "the move did change the board");

    for step in 1..=10 {
        app.update(None, millis(50));
        assert_eq!(
            app.shown_score(),
            app.game.score,
            "shown must equal real after {step} * 50 ms"
        );
    }
}

#[test]
fn score_count_does_not_touch_the_game() {
    let mut counted = sixteen_point_app();
    let mut frozen = sixteen_point_app();

    counted.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    frozen.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    for _ in 0..8 {
        counted.update(None, millis(50));
    }

    assert_ne!(
        counted.shown_score(),
        counted.game.score,
        "the ticker must still be counting for this test to mean anything"
    );
    assert_eq!(counted.game.score, frozen.game.score);
    assert_eq!(counted.game.best, frozen.game.best);
    assert_eq!(counted.game.board, frozen.game.board);
    assert_eq!(counted.game.rng.state(), frozen.game.rng.state());
    assert_eq!(counted.game.won, frozen.game.won);
    assert_eq!(counted.game.game_over, frozen.game.game_over);
}
