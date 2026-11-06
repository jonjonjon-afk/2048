//! M3 effect tests: injected clock, plasma, shockwave, sparks, flashes and pulse.
//!
//! Every animation frame is rendered from fixed state and fixed `dt`; nothing
//! here reads the wall clock or the real high-score file.

use std::collections::HashSet;
use std::time::Duration;

use game2048::demo::demo_merge_app;
use game2048::effects::{RING_SYMBOLS, SPARK_SYMBOLS};
use game2048::game::{Board, Direction, Game};
use game2048::render::{
    board_layout, frame_buffer, frame_text, header_style, tile_colour, tile_style, BOARD_HEIGHT,
    BOARD_WIDTH, CARD_GAP, CARD_HEIGHT, CARD_WIDTH,
};
use game2048::update::App;
use game2048::Action;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

fn cell_style(buffer: &Buffer, x: u16, y: u16) -> Style {
    let cell = &buffer[(x, y)];
    Style::new()
        .fg(cell.fg)
        .bg(cell.bg)
        .add_modifier(cell.modifier)
}

fn inner_cell_rect(cell: usize, width: u16, height: u16) -> Rect {
    let layout = board_layout(width, height);
    let row = (cell / 4) as u16;
    let column = (cell % 4) as u16;
    Rect::new(
        layout.left + column * (CARD_WIDTH + CARD_GAP) + 1,
        layout.top + row * (CARD_HEIGHT + CARD_GAP) + 1,
        CARD_WIDTH - 2,
        CARD_HEIGHT - 2,
    )
}

fn find_symbol(buffer: &Buffer, area: Rect, target: char) -> Option<(u16, u16)> {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if buffer[(x, y)].symbol().starts_with(target) {
                return Some((x, y));
            }
        }
    }
    None
}

fn find_token(buffer: &Buffer, y: u16, token: &str) -> Option<(u16, u16)> {
    let line: Vec<char> = (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol().chars().next().unwrap_or(' '))
        .collect();
    let needle: Vec<char> = token.chars().collect();
    (0..=line.len().saturating_sub(needle.len()))
        .find(|start| line[*start..*start + needle.len()] == needle[..])
        .map(|start| (start as u16, y))
}

fn ring_positions(buffer: &Buffer) -> Vec<(u16, u16, char)> {
    let mut positions = Vec::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if let Some(character) = buffer[(x, y)].symbol().chars().next() {
                if RING_SYMBOLS.contains(&character) {
                    positions.push((x, y, character));
                }
            }
        }
    }
    positions
}

fn chebyshev_from_cell(x: u16, y: u16, cell: usize, width: u16, height: u16) -> i32 {
    let layout = board_layout(width, height);
    let column = (cell % 4) as u16;
    let row = (cell / 4) as u16;
    let centre_x = layout.left + column * (CARD_WIDTH + CARD_GAP) + CARD_WIDTH / 2;
    let centre_y = layout.top + row * (CARD_HEIGHT + CARD_GAP) + CARD_HEIGHT / 2;
    (i32::from(x) - i32::from(centre_x))
        .abs()
        .max((i32::from(y) - i32::from(centre_y)).abs())
}

fn max_ring_distance(buffer: &Buffer, cell: usize) -> i32 {
    ring_positions(buffer)
        .iter()
        .map(|(x, y, _)| chebyshev_from_cell(*x, *y, cell, 100, 36))
        .max()
        .unwrap_or(0)
}

fn assert_buffers_identical(first: &Buffer, second: &Buffer) {
    assert_eq!(first.area, second.area);
    for y in 0..first.area.height {
        for x in 0..first.area.width {
            let a = &first[(x, y)];
            let b = &second[(x, y)];
            assert_eq!(a.symbol(), b.symbol(), "symbol at {x},{y}");
            assert_eq!(a.fg, b.fg, "foreground at {x},{y}");
            assert_eq!(a.bg, b.bg, "background at {x},{y}");
            assert_eq!(a.modifier, b.modifier, "modifier at {x},{y}");
        }
    }
}

#[test]
fn frame_n_advances_effects_by_n_times_16ms() {
    for (frame, millis) in [(0u64, 0u64), (4, 64), (14, 224), (24, 384)] {
        let app = demo_merge_app(frame);
        assert_eq!(
            app.effects.elapsed(),
            Duration::from_millis(millis),
            "frame {frame} must be {millis} ms"
        );
    }
}

#[test]
fn one_100ms_step_equals_ten_10ms_steps() {
    let mut one_step = demo_merge_app(0);
    one_step.update(None, Duration::from_millis(100));

    let mut ten_steps = demo_merge_app(0);
    for _ in 0..10 {
        ten_steps.update(None, Duration::from_millis(10));
    }

    assert_eq!(one_step.effects, ten_steps.effects);
    assert_eq!(one_step.effects.elapsed(), Duration::from_millis(100));
}

#[test]
fn effects_do_not_advance_the_game_rng() {
    let mut app = demo_merge_app(0);
    let state = app.game.rng.state();

    for _ in 0..40 {
        app.update(None, Duration::from_millis(16));
    }

    assert_eq!(app.game.rng.state(), state);
}

#[test]
fn effects_do_not_change_the_game_state() {
    let mut app = App::new(Game::new(42, 0));
    let mut plain = Game::new(42, 0);
    let directions = [
        Direction::Left,
        Direction::Down,
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
        Direction::Up,
    ];

    for direction in directions {
        app.update(Some(Action::Move(direction)), Duration::from_millis(16));
        plain.apply_move(direction);

        assert_eq!(app.game.board, plain.board);
        assert_eq!(app.game.score, plain.score);
        assert_eq!(app.game.best, plain.best);
        assert_eq!(app.game.won, plain.won);
        assert_eq!(app.game.game_over, plain.game_over);
        assert_eq!(app.game.rng.state(), plain.rng.state());
    }
}

#[test]
fn plasma_covers_the_frame_with_truecolour() {
    let buffer = frame_buffer(&App::new(Game::new(42, 0)), 100, 36);

    for y in 0..36 {
        for x in 0..100 {
            let cell = &buffer[(x, y)];
            assert!(
                matches!(cell.fg, Color::Rgb(..)),
                "foreground at {x},{y} is {:?}",
                cell.fg
            );
            assert!(
                matches!(cell.bg, Color::Rgb(..)),
                "background at {x},{y} is {:?}",
                cell.bg
            );
        }
    }
}

#[test]
fn plasma_phase_advances_with_dt() {
    let before = frame_buffer(&App::new(Game::new(42, 0)), 100, 36);
    let mut app = App::new(Game::new(42, 0));
    app.update(None, Duration::from_millis(100));
    let after = frame_buffer(&app, 100, 36);

    let changed = (0..36).any(|y| {
        (0..100)
            .any(|x| before[(x, y)].fg != after[(x, y)].fg || before[(x, y)].bg != after[(x, y)].bg)
    });
    assert!(changed, "plasma must change between 0 ms and 100 ms");
}

#[test]
fn plasma_is_deterministic() {
    let first = frame_buffer(&demo_merge_app(14), 100, 36);
    let second = frame_buffer(&demo_merge_app(14), 100, 36);

    assert_buffers_identical(&first, &second);
}

#[test]
fn shockwave_uses_ring_characters() {
    let mut seen = HashSet::new();
    for frame in [4, 14, 24] {
        let buffer = frame_buffer(&demo_merge_app(frame), 100, 36);
        for (_, _, character) in ring_positions(&buffer) {
            seen.insert(character);
        }
    }

    for character in RING_SYMBOLS {
        assert!(
            seen.contains(&character),
            "ring character {character:?} is missing"
        );
    }
}

#[test]
fn shockwave_expands_and_expires_after_450ms() {
    let frame_4 = frame_buffer(&demo_merge_app(4), 100, 36);
    let frame_14 = frame_buffer(&demo_merge_app(14), 100, 36);
    let frame_24 = frame_buffer(&demo_merge_app(24), 100, 36);

    let radius_4 = max_ring_distance(&frame_4, 0);
    let radius_14 = max_ring_distance(&frame_14, 0);
    let radius_24 = max_ring_distance(&frame_24, 0);
    assert!(
        radius_4 < radius_14 && radius_14 < radius_24,
        "radii {radius_4}, {radius_14}, {radius_24}"
    );

    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(449));
    assert_eq!(app.effects.shockwaves().len(), 1, "active at 449 ms");
    assert!(
        !ring_positions(&frame_buffer(&app, 100, 36)).is_empty(),
        "ring is rendered at 449 ms"
    );

    app.update(None, Duration::from_millis(1));
    assert!(app.effects.shockwaves().is_empty(), "gone at 450 ms");
    assert!(
        ring_positions(&frame_buffer(&app, 100, 36)).is_empty(),
        "no ring is rendered at 450 ms"
    );
}

#[test]
fn shockwave_snapshots_at_frames_4_14_24() {
    assert_eq!(
        frame_text(&demo_merge_app(4), 100, 36),
        include_str!("fixtures/effect_100x36_frame_04.txt")
    );
    assert_eq!(
        frame_text(&demo_merge_app(14), 100, 36),
        include_str!("fixtures/effect_100x36_frame_14.txt")
    );
    assert_eq!(
        frame_text(&demo_merge_app(24), 100, 36),
        include_str!("fixtures/effect_100x36_frame_24.txt")
    );
}

#[test]
fn shockwave_colour_comes_from_the_merged_value() {
    for frame in [4, 14, 24] {
        let buffer = frame_buffer(&demo_merge_app(frame), 100, 36);
        let positions = ring_positions(&buffer);
        assert!(!positions.is_empty(), "frame {frame} has no ring");

        assert!(
            positions
                .iter()
                .any(|(x, y, _)| buffer[(*x, *y)].fg == tile_colour(4)),
            "frame {frame} has no ring cell in the value-4 tile colour"
        );
    }
}

#[test]
fn sparks_are_fourteen_per_merge_and_expire_after_500ms() {
    let mut app = demo_merge_app(0);
    assert_eq!(app.effects.sparks().len(), 14);

    app.update(None, Duration::from_millis(499));
    assert_eq!(app.effects.sparks().len(), 14, "alive at 499 ms");

    app.update(None, Duration::from_millis(1));
    assert!(app.effects.sparks().is_empty(), "gone at 500 ms");
}

#[test]
fn sparks_stay_inside_the_board() {
    let layout = board_layout(100, 36);
    let left = layout.left;
    let top = layout.top;

    for frame in [4, 14, 24] {
        let buffer = frame_buffer(&demo_merge_app(frame), 100, 36);
        let mut visible = 0;
        for y in 0..36 {
            for x in 0..100 {
                if let Some(character) = buffer[(x, y)].symbol().chars().next() {
                    if SPARK_SYMBOLS.contains(&character) {
                        visible += 1;
                        assert!(
                            x >= left
                                && x < left + BOARD_WIDTH
                                && y >= top
                                && y < top + BOARD_HEIGHT,
                            "spark at {x},{y} is outside the board block"
                        );
                    }
                }
            }
        }
        assert!(visible > 0, "frame {frame} renders no spark");
    }
}

#[test]
fn sparks_are_deterministic() {
    let first = demo_merge_app(14);
    let second = demo_merge_app(14);

    assert_eq!(first.effects.sparks(), second.effects.sparks());
    assert_eq!(frame_text(&first, 100, 36), frame_text(&second, 100, 36));
}

#[test]
fn sparks_use_the_four_symbols() {
    let app = demo_merge_app(0);
    let used: HashSet<char> = app
        .effects
        .sparks()
        .iter()
        .map(|spark| spark.symbol)
        .collect();

    for character in SPARK_SYMBOLS {
        assert!(
            used.contains(&character),
            "spark symbol {character:?} is unused"
        );
    }
}

#[test]
fn tile_flash_lasts_180ms() {
    for millis in [0u64, 179] {
        let mut app = demo_merge_app(0);
        app.update(None, Duration::from_millis(millis));
        let buffer = frame_buffer(&app, 100, 36);
        let position = find_symbol(&buffer, inner_cell_rect(0, 100, 36), '4')
            .expect("the merged cell must show 4");

        assert_ne!(
            cell_style(&buffer, position.0, position.1),
            tile_style(4),
            "flash must be visible at {millis} ms"
        );
    }

    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(180));
    let buffer = frame_buffer(&app, 100, 36);
    let position = find_symbol(&buffer, inner_cell_rect(0, 100, 36), '4')
        .expect("the merged cell must show 4");
    assert_eq!(
        cell_style(&buffer, position.0, position.1),
        tile_style(4),
        "flash must be gone at 180 ms"
    );
}

#[test]
fn spawn_pop_lasts_120ms() {
    for millis in [0u64, 119] {
        let mut app = demo_merge_app(0);
        app.update(None, Duration::from_millis(millis));
        let buffer = frame_buffer(&app, 100, 36);
        let position = find_symbol(&buffer, inner_cell_rect(9, 100, 36), '2')
            .expect("the spawned cell must show 2");

        assert_ne!(
            cell_style(&buffer, position.0, position.1),
            tile_style(2),
            "pop must be visible at {millis} ms"
        );
    }

    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(120));
    let buffer = frame_buffer(&app, 100, 36);
    let position = find_symbol(&buffer, inner_cell_rect(9, 100, 36), '2')
        .expect("the spawned cell must show 2");
    assert_eq!(
        cell_style(&buffer, position.0, position.1),
        tile_style(2),
        "pop must be gone at 120 ms"
    );
}

#[test]
fn flash_and_pop_do_not_change_the_board() {
    let mut app = demo_merge_app(0);
    let board = app.game.board;
    let score = app.game.score;
    let best = app.game.best;
    let rng = app.game.rng.state();

    app.update(None, Duration::from_millis(179));

    assert_eq!(app.game.board, board);
    assert_eq!(app.game.score, score);
    assert_eq!(app.game.best, best);
    assert_eq!(app.game.rng.state(), rng);
}

#[test]
fn best_pulse_lasts_600ms() {
    for millis in [0u64, 300, 599] {
        let mut app = demo_merge_app(0);
        app.update(None, Duration::from_millis(millis));
        let buffer = frame_buffer(&app, 100, 36);
        let position = find_token(&buffer, 0, "BEST").expect("header shows BEST");

        assert_ne!(
            cell_style(&buffer, position.0, position.1),
            header_style(),
            "BEST must pulse at {millis} ms"
        );
    }

    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(600));
    let buffer = frame_buffer(&app, 100, 36);
    let position = find_token(&buffer, 0, "BEST").expect("header shows BEST");
    assert_eq!(
        cell_style(&buffer, position.0, position.1),
        header_style(),
        "BEST must be normal at 600 ms"
    );
}

#[test]
fn best_pulse_starts_only_on_a_new_best() {
    let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);

    let mut same_best = App::new(Game::from_state(board, 0, 4, 42));
    same_best.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert_eq!(same_best.game.score, 4);
    assert_eq!(same_best.game.best, 4);
    assert_eq!(same_best.effects.best_pulse_age(), None);

    let mut new_best = App::new(Game::from_state(board, 0, 3, 42));
    new_best.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert_eq!(new_best.game.best, 4);
    assert_eq!(new_best.effects.best_pulse_age(), Some(Duration::ZERO));
}
