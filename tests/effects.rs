//! M7 effect tests: injected clock, plasma, merge ripple, sparks, flashes and pulse.
//!
//! Every animation frame is rendered from fixed state and fixed `dt`; nothing
//! here reads the wall clock or the real high-score file.

use std::collections::HashSet;
use std::time::Duration;

use game2048::demo::{demo_merge_app, demo_slide_app};
use game2048::effects::{
    Effects, BEST_PULSE_MILLIS, CELL_CENTRE_X, CELL_CENTRE_Y, CELL_PITCH_X, CELL_PITCH_Y,
    FLASH_MILLIS, POP_MILLIS, RIPPLE_BOTTOM_GLYPH, RIPPLE_FRONT_WINDOW, RIPPLE_LIT_THRESHOLD,
    RIPPLE_ROW_ASPECT, RIPPLE_TOP_GLYPH, RIPPLE_WALL_FACTOR, SPARKS_PER_MERGE, SPARK_MILLIS,
    SPARK_SYMBOLS,
};
use game2048::game::{Board, Direction, Game, Merge};
use game2048::highscore::HighScores;
use game2048::render::{
    board_layout, frame_buffer, frame_text, header_style, panel_layout, tile_style, BOARD_HEIGHT,
    BOARD_WIDTH, CARD_GAP, CARD_HEIGHT, CARD_WIDTH,
};
use game2048::update::App;
use game2048::Action;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

const WIDTH: u16 = 100;
const HEIGHT: u16 = 36;
const PLAY_WIDTH: i32 = 39;
const PLAY_HEIGHT: i32 = 23;

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

/// Sum of the RGB channels; a cheap "how bright" measure for the fade test.
fn color_luma(colour: Color) -> u32 {
    match colour {
        Color::Rgb(red, green, blue) => u32::from(red) + u32::from(green) + u32::from(blue),
        _ => 0,
    }
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

/// The demo merge advanced by an arbitrary exact age (frames are 16 ms steps).
fn app_at(millis: u64) -> App {
    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(millis));
    app
}

/// Read one play-area cell exactly where the renderer draws it.
///
/// Play-area cell `(x, y)` sits at `panel.left + 1 + x`, `panel.top + 1 + y`.
fn play_cell(buffer: &Buffer, x: i32, y: i32) -> char {
    let panel = panel_layout(WIDTH, HEIGHT);
    let column = panel.left + 1 + x as u16;
    let row = panel.top + 1 + y as u16;
    buffer[(column, row)].symbol().chars().next().unwrap_or(' ')
}

/// True for the two half-block glyphs the M11 ripple renders.
fn is_ripple_glyph(glyph: char) -> bool {
    glyph == RIPPLE_TOP_GLYPH || glyph == RIPPLE_BOTTOM_GLYPH
}

/// The foreground and background of one play-area cell, as drawn.
fn play_style(buffer: &Buffer, x: i32, y: i32) -> (Color, Color) {
    let panel = panel_layout(WIDTH, HEIGHT);
    let cell = &buffer[(panel.left + 1 + x as u16, panel.top + 1 + y as u16)];
    (cell.fg, cell.bg)
}

/// Every ripple glyph inside the 39x23 play area, with its play-area cell.
fn ripple_cells(buffer: &Buffer) -> Vec<(i32, i32, char)> {
    let mut cells = Vec::new();
    for y in 0..PLAY_HEIGHT {
        for x in 0..PLAY_WIDTH {
            let glyph = play_cell(buffer, x, y);
            if is_ripple_glyph(glyph) {
                cells.push((x, y, glyph));
            }
        }
    }
    cells
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
        // M13: 150 ms completes the slide inside this update, so the next move
        // is not swallowed by the 140 ms move lock.
        app.update(Some(Action::Move(direction)), Duration::from_millis(150));
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

    assert_ne!(
        before[(2, 2)].bg,
        after[(2, 2)].bg,
        "plasma background at the non-UI cell 2,2 must change between 0 ms and 100 ms"
    );
}

#[test]
fn plasma_background_reaches_the_frame_buffer() {
    let before = frame_buffer(&App::new(Game::new(42, 0)), 100, 36);
    let mut app = App::new(Game::new(42, 0));
    app.update(None, Duration::from_millis(100));
    let after = frame_buffer(&app, 100, 36);

    let mut backgrounds = Vec::new();
    for x in 1..99 {
        let background = before[(x, 2)].bg;
        assert!(
            matches!(background, Color::Rgb(..)),
            "blank-row background at {x},2 is {background:?}"
        );
        if !backgrounds.contains(&background) {
            backgrounds.push(background);
        }
    }

    assert!(
        backgrounds.len() >= 2,
        "blank row 2 has only {} distinct Rgb backgrounds; the plasma did not reach the buffer",
        backgrounds.len()
    );
    assert_ne!(
        before[(2, 2)].bg,
        after[(2, 2)].bg,
        "the sampled background must shimmer between 0 ms and 100 ms"
    );
}

#[test]
fn plasma_is_deterministic() {
    let first = frame_buffer(&demo_merge_app(14), 100, 36);
    let second = frame_buffer(&demo_merge_app(14), 100, 36);

    assert_buffers_identical(&first, &second);
}

#[test]
fn ripple_radius_matches_the_spec_at_every_pin() {
    // SPEC 5.4: radius = age_ms as f64 * 0.04, no rounding.
    for (millis, expected) in [
        (100u64, 4.0f64),
        (150, 6.0),
        (224, 8.96),
        (480, 19.2),
        (700, 28.0),
        (880, 35.2),
    ] {
        let app = app_at(millis);
        let ripples = app.effects.ripples();
        assert_eq!(ripples.len(), 1, "{millis} ms has one active ripple");
        assert_eq!(ripples[0].radius(), expected, "{millis} ms radius");
        assert!(
            (ripples[0].amplitude() - (1.0 - millis as f64 / 900.0)).abs() < f64::EPSILON,
            "{millis} ms base amplitude"
        );
    }

    // The ripple geometry stays tied to the 9x5 cards with 1-cell gaps.
    assert_eq!(CELL_PITCH_X, f64::from(CARD_WIDTH + CARD_GAP));
    assert_eq!(CELL_PITCH_Y, f64::from(CARD_HEIGHT + CARD_GAP));
    assert_eq!(CELL_CENTRE_X, f64::from(CARD_WIDTH) / 2.0);
    assert_eq!(CELL_CENTRE_Y, f64::from(CARD_HEIGHT) / 2.0);
    assert_eq!(RIPPLE_FRONT_WINDOW, 0.75);
    assert_eq!(RIPPLE_WALL_FACTOR, 0.55);
}

/// One M11 hand-table row: age, play-area cell, drawn glyph, top/bottom amp.
type SubcellPin = (u64, i32, i32, Option<char>, f64, f64);

#[test]
fn ripple_halves_match_the_subcell_table() {
    // M11 hand table. `None` means the frame cell carries no ripple glyph; a
    // digit or a spark may still own that cell (the label layer draws last).
    let table: &[SubcellPin] = &[
        (100, 0, 2, Some('▀'), 0.888889, 0.888889),
        (100, 5, 5, Some('▀'), 0.888889, 0.0),
        (100, 3, 0, Some('▄'), 0.0, 0.888889),
        (100, 0, 4, None, 0.0, 0.0),
        (100, 4, 2, None, 0.0, 0.0),
        (100, 20, 12, None, 0.0, 0.0),
        (150, 10, 2, Some('▀'), 0.833333, 0.833333),
        (150, 0, 2, Some('▀'), 0.458333, 0.458333),
        (150, 0, 0, Some('▄'), 0.0, 0.833333),
        (224, 13, 2, Some('▀'), 0.751111, 0.751111),
        (224, 2, 5, Some('▀'), 0.413111, 0.413111),
        (224, 7, 1, Some('▀'), 0.413111, 0.0),
        (224, 2, 4, Some('▄'), 0.0, 0.413111),
        (224, 13, 10, None, 0.0, 0.0),
        (480, 23, 2, Some('▀'), 0.466667, 0.466667),
        (480, 13, 2, Some('▀'), 0.256667, 0.256667),
        (480, 9, 2, Some('▄'), 0.0, 0.141167),
        (480, 5, 5, Some('▀'), 0.141167, 0.0),
        (700, 32, 2, Some('▀'), 0.222222, 0.222222),
        (700, 13, 10, Some('▀'), 0.122222, 0.0),
        (700, 23, 2, Some('▀'), 0.122222, 0.122222),
    ];

    assert_eq!(RIPPLE_LIT_THRESHOLD, 0.10, "a half is lit at >= 0.10");

    let mut current = u64::MAX;
    let mut ripple = app_at(0).effects.ripples()[0];
    let mut buffer = frame_buffer(&app_at(0), WIDTH, HEIGHT);
    for &(millis, x, y, glyph, top, bottom) in table {
        if millis != current {
            let app = app_at(millis);
            ripple = app.effects.ripples()[0];
            buffer = frame_buffer(&app, WIDTH, HEIGHT);
            current = millis;
        }

        let halves = ripple.halves_at(x, y);
        assert!(
            (halves.top - top).abs() < 1e-6,
            "{millis} ms ({x},{y}) top amplitude {} != {top}",
            halves.top
        );
        assert!(
            (halves.bottom - bottom).abs() < 1e-6,
            "{millis} ms ({x},{y}) bottom amplitude {} != {bottom}",
            halves.bottom
        );
        let drawn = play_cell(&buffer, x, y);
        match glyph {
            Some(expected) => assert_eq!(drawn, expected, "{millis} ms ({x},{y}) glyph"),
            None => assert!(
                !is_ripple_glyph(drawn),
                "{millis} ms ({x},{y}) must carry no half block, found {drawn:?}"
            ),
        }
    }

    // At 880 ms the base amplitude is 0.0222, below every lit half.
    assert!(
        ripple_cells(&frame_buffer(&app_at(880), WIDTH, HEIGHT)).is_empty(),
        "880 ms must show no half block"
    );

    // R2: the old three glyph classes are gone from the whole frame.
    for millis in [100u64, 150, 224, 480, 700, 880] {
        let text = frame_text(&app_at(millis), WIDTH, HEIGHT);
        for stale in ['▓', '▒', '░'] {
            assert!(
                !text.contains(stale),
                "{millis} ms frame still contains {stale:?}"
            );
        }
    }
}

#[test]
fn ripple_front_at_100ms_matches_the_round_hand_table() {
    let buffer = frame_buffer(&app_at(100), WIDTH, HEIGHT);

    assert_eq!(
        play_cell(&buffer, 0, 2),
        '▀',
        "(0,2) is direct, both halves"
    );
    assert_eq!(
        play_cell(&buffer, 9, 2),
        '▀',
        "(9,2) is direct, both halves"
    );
    assert_eq!(play_cell(&buffer, 7, 1), '▀', "(7,1) is on the round front");
    assert_eq!(
        play_cell(&buffer, 5, 5),
        '▀',
        "(5,5) is top-half only (G19: the cell centre is off the front)"
    );
    assert_eq!(play_cell(&buffer, 3, 0), '▄', "(3,0) is bottom-half only");
    for (x, y) in [(0, 4), (4, 2), (20, 12)] {
        assert!(
            !is_ripple_glyph(play_cell(&buffer, x, y)),
            "({x},{y}) is off the round front"
        );
    }

    // The effect layer keeps the M10 cell-centre pins.
    let app = app_at(100);
    let ripple = app.effects.ripples()[0];
    assert_eq!(ripple.glyph_at(7, 1), Some('▓'));
    assert!(ripple.glyph_at(0, 4).is_none());
    assert!(ripple.glyph_at(0, 2).is_some());
    assert!(ripple.glyph_at(5, 5).is_none());
}

#[test]
fn ripple_is_round_not_square() {
    assert_eq!(RIPPLE_ROW_ASPECT, 2.0, "one row is two columns of length");

    // 100 ms: (7,1) d=3.905 is on the round front while (0,4) d=5.408 is off.
    // A Chebyshev front would light (0,4) at 4.5 and miss (7,1) at 2.5.
    let at_100 = app_at(100);
    let ripple = at_100.effects.ripples()[0];
    assert_eq!(ripple.glyph_at(7, 1), Some('▓'), "100 ms (7,1)");
    assert_eq!(ripple.glyph_at(0, 4), None, "100 ms (0,4)");
    let buffer = frame_buffer(&at_100, WIDTH, HEIGHT);
    assert_eq!(play_cell(&buffer, 7, 1), '▀', "100 ms (7,1) in the frame");
    assert!(
        !is_ripple_glyph(play_cell(&buffer, 0, 4)),
        "100 ms (0,4) is blank in the frame"
    );

    // 150 ms: the direct front reaches (0,0) at d=6.727; only the bottom
    // half-cell centre is within the window, so it draws a lower half block.
    let at_150 = app_at(150);
    assert_eq!(
        at_150.effects.ripples()[0].glyph_at(0, 0),
        Some('▓'),
        "150 ms (0,0)"
    );
    assert_eq!(
        play_cell(&frame_buffer(&at_150, WIDTH, HEIGHT), 0, 0),
        '▄',
        "150 ms (0,0) in the frame"
    );

    // 224 ms: (13,2) d=8.559 is on the crest; the old square pin (13,10) d=17.24
    // is now far off the radius 8.96.
    let at_224 = app_at(224);
    assert_eq!(
        at_224.effects.ripples()[0].glyph_at(13, 2),
        Some('▓'),
        "224 ms (13,2)"
    );
    assert_eq!(
        at_224.effects.ripples()[0].glyph_at(13, 10),
        None,
        "224 ms (13,10)"
    );
    let buffer = frame_buffer(&at_224, WIDTH, HEIGHT);
    assert_eq!(play_cell(&buffer, 13, 2), '▀', "224 ms (13,2) in the frame");
    assert!(
        !is_ripple_glyph(play_cell(&buffer, 13, 10)),
        "224 ms (13,10) is blank in the frame"
    );
}

#[test]
fn ripple_reflects_off_the_left_wall() {
    let at_150 = frame_buffer(&app_at(150), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_150, 10, 2), '▀', "(10,2) direct at 150 ms");
    assert_eq!(
        play_cell(&at_150, 0, 2),
        '▀',
        "(0,2) is the left-wall image at 150 ms"
    );

    let at_224 = app_at(224);
    let ripple = at_224.effects.ripples()[0];
    let (glyph, amplitude) = ripple
        .front_at(3, 2)
        .expect("(3,2) is on the left-wall image at 224 ms");
    assert_eq!(glyph, '▒', "(3,2) effect-layer reflected crest");
    assert!(
        (amplitude - 0.4131111111111111).abs() < 1e-12,
        "(3,2) amplitude is 0.413111, got {amplitude}"
    );

    let at_224 = frame_buffer(&at_224, WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_224, 13, 2), '▀', "(13,2) direct at 224 ms");
    assert_eq!(
        play_cell(&at_224, 2, 5),
        '▀',
        "(2,5) is the rendered left-wall image at 224 ms"
    );
    assert_eq!(
        play_cell(&at_224, 7, 1),
        '▀',
        "(7,1) is the top-wall image at 224 ms"
    );
    assert_eq!(
        play_cell(&at_224, 2, 4),
        '▄',
        "(2,4) is the bottom-half-only left-wall image at 224 ms"
    );
    assert!(
        !is_ripple_glyph(play_cell(&at_224, 9, 2)),
        "(9,2) sits in the gap between the direct and reflected crests"
    );
}

#[test]
fn ripple_fades_out_and_expires_at_900ms() {
    let at_480 = frame_buffer(&app_at(480), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_480, 23, 2), '▀', "(23,2) at 480 ms");
    assert_eq!(
        play_cell(&at_480, 13, 2),
        '▀',
        "(13,2) is the left-wall image at 480 ms"
    );
    assert_eq!(
        play_cell(&at_480, 9, 2),
        '▄',
        "(9,2) is the two-wall bottom half at 480 ms"
    );
    assert_eq!(
        play_cell(&at_480, 5, 5),
        '▀',
        "(5,5) is the two-wall top half at 480 ms"
    );

    let at_700 = frame_buffer(&app_at(700), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_700, 32, 2), '▀', "(32,2) at 700 ms");
    assert_eq!(
        play_cell(&at_700, 13, 10),
        '▀',
        "(13,10) is the top-wall image at 700 ms"
    );
    assert_eq!(
        play_cell(&at_700, 23, 2),
        '▀',
        "(23,2) is the left-wall image at 700 ms"
    );

    let at_880 = frame_buffer(&app_at(880), WIDTH, HEIGHT);
    assert!(
        ripple_cells(&at_880).is_empty(),
        "880 ms must show no ripple glyph, found {:?}",
        ripple_cells(&at_880)
    );

    let mut app = app_at(899);
    assert_eq!(app.effects.ripples().len(), 1, "ripple is alive at 899 ms");
    app.update(None, Duration::from_millis(1));
    assert!(
        app.effects.ripples().is_empty(),
        "the active-wave accessor is empty at 900 ms"
    );
    assert!(ripple_cells(&frame_buffer(&app, WIDTH, HEIGHT)).is_empty());
}

#[test]
fn a_merge_starts_one_ripple_one_flash_and_fourteen_sparks() {
    let app = demo_merge_app(0);
    let ripples = app.effects.ripples();

    assert_eq!(ripples.len(), 1, "one merge starts exactly one ripple");
    assert_eq!(ripples[0].cell, 0);
    assert_eq!(ripples[0].value, 4);
    assert_eq!(app.effects.sparks().len(), SPARKS_PER_MERGE);
    assert!(app.effects.is_flashing(0), "the merged cell flashes");
    assert!(!app.effects.is_popping(0));

    assert_eq!(SPARKS_PER_MERGE, 14);
    assert_eq!(SPARK_MILLIS, 500);
    assert_eq!(FLASH_MILLIS, 180);
    assert_eq!(POP_MILLIS, 120);
    assert_eq!(BEST_PULSE_MILLIS, 600);
}

#[test]
fn two_merges_keep_two_waves() {
    let mut effects = Effects::new();
    effects.trigger_merge(Merge { cell: 0, value: 4 });
    effects.trigger_merge(Merge { cell: 5, value: 8 });

    let cells: Vec<usize> = effects.ripples().iter().map(|ripple| ripple.cell).collect();
    let values: Vec<u32> = effects
        .ripples()
        .iter()
        .map(|ripple| ripple.value)
        .collect();
    assert_eq!(cells, vec![0, 5], "each merge keeps its own source cell");
    assert_eq!(values, vec![4, 8], "each merge keeps its own value");
    assert_eq!(effects.sparks().len(), SPARKS_PER_MERGE * 2);

    effects.advance(Duration::from_millis(899));
    assert_eq!(
        effects.ripples().len(),
        2,
        "both waves live while age < 900"
    );
    effects.advance(Duration::from_millis(1));
    assert!(effects.ripples().is_empty(), "both waves expire at 900 ms");
}

#[test]
fn one_move_with_two_merges_keeps_two_waves() {
    let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [2, 2, 0, 0]]);
    let mut app = App::new(Game::from_state(board, 0, 0, 42));
    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    // M13: the ripples start at the landing, not on the keypress.
    app.update(None, Duration::from_millis(140));

    let cells: Vec<usize> = app
        .effects
        .ripples()
        .iter()
        .map(|ripple| ripple.cell)
        .collect();
    let values: Vec<u32> = app
        .effects
        .ripples()
        .iter()
        .map(|ripple| ripple.value)
        .collect();
    assert_eq!(cells, vec![0, 12], "one ripple per merge, in move order");
    assert_eq!(values, vec![4, 4]);
}

#[test]
fn tile_digits_stay_on_top_of_the_ripple() {
    // Frame 1: radius 0.64 covers the merged card's centre cell (4,2), which is
    // also where cell 0's `4` digit is drawn. The label layer must win.
    let frame_1 = demo_merge_app(1);
    let ripple = frame_1.effects.ripples()[0];
    assert_eq!(
        ripple.glyph_at(4, 2),
        Some('▓'),
        "the direct front must cover the merged card's centre at 16 ms"
    );
    assert_eq!(
        play_cell(&frame_buffer(&frame_1, WIDTH, HEIGHT), 4, 2),
        '4',
        "frame 1 must keep the digit at (4,2), not the ripple glyph"
    );

    // Frame 2 still reads the digit.
    assert_eq!(
        play_cell(&frame_buffer(&demo_merge_app(2), WIDTH, HEIGHT), 4, 2),
        '4',
        "frame 2 must keep the digit at (4,2)"
    );

    // Frame 14: the ripple is still drawn over the cards where no label sits.
    let buffer = frame_buffer(&demo_merge_app(14), WIDTH, HEIGHT);
    assert_eq!(play_cell(&buffer, 13, 2), '▀', "frame 14 ripple at (13,2)");
    let ripple = demo_merge_app(14).effects.ripples()[0];
    assert_eq!(
        ripple.glyph_at(4, 2),
        Some('▒'),
        "at 224 ms the left-wall image covers (4,2), so the label still wins"
    );
    assert_eq!(play_cell(&buffer, 4, 2), '4', "frame 14 keeps the digit");
}

#[test]
fn ripple_colour_fades_with_age() {
    let style_at = |millis: u64, x: i32, y: i32| -> (Color, Color) {
        play_style(&frame_buffer(&app_at(millis), WIDTH, HEIGHT), x, y)
    };
    let background = Color::Rgb(6, 8, 20);

    // SPEC 5.4 Brightness: fg = bg + (colour - bg) * amplitude, rounded, with
    // bg = Rgb(6, 8, 20) and the value-4 colour Rgb(0, 200, 255). A both-lit
    // cell has fg = top colour and bg = bottom colour.
    let direct_224 = Color::Rgb(1, 152, 197);
    assert_eq!(
        style_at(224, 13, 2),
        (direct_224, direct_224),
        "224 ms direct crest, both halves"
    );
    let reflected_224 = Color::Rgb(4, 87, 117);
    assert_eq!(
        style_at(224, 2, 5),
        (reflected_224, reflected_224),
        "224 ms left-wall crest, both halves"
    );
    let direct_480 = Color::Rgb(3, 98, 130);
    assert_eq!(
        style_at(480, 23, 2),
        (direct_480, direct_480),
        "480 ms direct crest, both halves"
    );
    let direct_700 = Color::Rgb(5, 51, 72);
    assert_eq!(
        style_at(700, 32, 2),
        (direct_700, direct_700),
        "700 ms direct crest, both halves"
    );

    // One-half cells: the lit half is the foreground, the unlit half is the
    // renderer background.
    assert_eq!(
        style_at(224, 7, 1),
        (reflected_224, background),
        "224 ms top-half-only wall crest"
    );
    assert_eq!(
        style_at(100, 3, 0),
        (Color::Rgb(1, 179, 229), background),
        "100 ms bottom-half-only direct crest"
    );

    // A two-colour cell: top = the 150 ms left-wall image (0.458333), bottom
    // = the direct image (0.833333); these are the reviewer's round-2
    // corrected literals (Q16).
    assert_eq!(
        style_at(150, 0, 4),
        (Color::Rgb(3, 96, 128), Color::Rgb(1, 168, 216)),
        "150 ms two-colour cell"
    );

    assert!(
        color_luma(direct_224) > color_luma(reflected_224),
        "the 224 ms direct crest must be brighter than the reflected crest"
    );
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
    let row = panel_layout(100, 36).top;
    for millis in [0u64, 300, 599] {
        let mut app = demo_merge_app(0);
        app.update(None, Duration::from_millis(millis));
        let buffer = frame_buffer(&app, 100, 36);
        let position = find_token(&buffer, row, "BEST").expect("the play-area top edge shows BEST");

        assert_ne!(
            cell_style(&buffer, position.0, position.1),
            header_style(),
            "BEST must pulse at {millis} ms"
        );
    }

    let mut app = demo_merge_app(0);
    app.update(None, Duration::from_millis(600));
    let buffer = frame_buffer(&app, 100, 36);
    let position = find_token(&buffer, row, "BEST").expect("the play-area top edge shows BEST");
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

// ---------------------------------------------------------------------------
// M13: slide frames, jiggle frames and overlays waiting for the landing.
// ---------------------------------------------------------------------------

#[test]
fn a_sliding_card_is_drawn_between_its_slots() {
    let app = demo_slide_app(3);
    assert!(app.sliding(), "frame 3 is inside the slide");
    let buffer = frame_buffer(&app, WIDTH, HEIGHT);

    // The two 2s of column 2 travel from rows 0/1 toward row 3; at 48 ms their
    // labels sit at play cells (24,12) and (24,15) (R4 pins).
    assert_eq!(play_cell(&buffer, 24, 12), '2', "cell 2's moving 2");
    assert_eq!(play_cell(&buffer, 24, 15), '2', "cell 6's moving 2");
}

#[test]
fn the_merged_value_waits_for_the_landing() {
    let during = frame_buffer(&demo_slide_app(3), WIDTH, HEIGHT);
    assert_ne!(
        play_cell(&during, 24, 20),
        '4',
        "the merged 4 must not be drawn during the slide"
    );

    let after = frame_buffer(&demo_slide_app(9), WIDTH, HEIGHT);
    assert_eq!(
        play_cell(&after, 24, 18),
        '4',
        "the merged 4 appears at the landing, two rows up along the merge axis"
    );
}

#[test]
fn the_spawned_tile_is_hidden_during_the_slide() {
    let during = demo_slide_app(3);
    let spawn = during.game.last_spawn.expect("the demo move spawns");
    let origin_x = (spawn.cell % 4) as i32 * 10;
    let origin_y = (spawn.cell / 4) as i32 * 6;

    let buffer = frame_buffer(&during, WIDTH, HEIGHT);
    for y in origin_y..origin_y + 5 {
        for x in origin_x..origin_x + 9 {
            let symbol = play_cell(&buffer, x, y);
            assert!(
                !symbol.is_ascii_digit(),
                "the spawn slot at ({x},{y}) shows {symbol:?} during the slide"
            );
        }
    }

    // F1: derive the spawn cell and its label position from `last_spawn` rather
    // than pinning a demo seed that moves the tile. After the landing the value
    // sits on card row 2 under the `(7 - digits) / 2` centring.
    let after = frame_buffer(&demo_slide_app(9), WIDTH, HEIGHT);
    let value = spawn.value.to_string();
    let padding = (7 - value.chars().count() as i32) / 2;
    for (offset, digit) in value.chars().enumerate() {
        let x = origin_x + 1 + padding + offset as i32;
        assert_eq!(
            play_cell(&after, x, origin_y + 2),
            digit,
            "the spawned tile is drawn after the landing"
        );
    }
}

#[test]
fn the_moving_plaque_is_opaque() {
    let background = Color::Rgb(6, 8, 20);

    let during = frame_buffer(&demo_slide_app(3), WIDTH, HEIGHT);
    assert_eq!(
        play_style(&during, 24, 11).1,
        background,
        "the moving card's interior is the renderer background"
    );

    let after = frame_buffer(&demo_slide_app(9), WIDTH, HEIGHT);
    assert_ne!(
        play_style(&after, 24, 11).1,
        background,
        "after the landing the same cell shows the plasma"
    );
}

#[test]
fn a_merged_card_jiggles_by_the_documented_offsets() {
    // The pinned demo frames: landing at 144 ms, so jiggle ages 0/48/96/176/240.
    // The move is Down, so the wobble is vertical: the label row moves while
    // the anchor's row clamps at 18.
    for (frame, expected) in [
        (9u64, (24u16, 18u16)),
        (12, (24, 20)),
        (15, (24, 19)),
        (20, (24, 20)),
        (24, (24, 20)),
    ] {
        let buffer = frame_buffer(&demo_slide_app(frame), WIDTH, HEIGHT);
        assert_eq!(
            play_cell(&buffer, i32::from(expected.0), i32::from(expected.1)),
            '4',
            "frame {frame} merged 4"
        );
    }

    // The 8 8 0 0 left merge at cell 0: dx = +2, -2 (clamped), +1.
    let board = Board::from_rows([[8, 8, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    let mut app = App::new(Game::from_state(board, 0, 0, 42));
    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    app.update(None, Duration::from_millis(140));

    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 5, 2),
        '1',
        "age 0: dx = +2"
    );
    app.update(None, Duration::from_millis(40));
    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 3, 2),
        '1',
        "age 40: dx = -2 clamped to x = 0"
    );
    app.update(None, Duration::from_millis(40));
    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 4, 2),
        '1',
        "age 80: dx = +1"
    );
}

#[test]
fn a_down_move_jiggles_up_first() {
    // The demo-slide board moved Down: merged cell 14 (anchor 20,18).
    let mut app = demo_slide_app(9);
    assert_eq!(app.jiggle_offset(14), Some((0, -2)), "the first beat is -y");
    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 24, 18),
        '4',
        "the label moves up with the card"
    );

    app.update(None, Duration::from_millis(40));
    assert_eq!(app.jiggle_offset(14), Some((0, 2)), "the second beat is +y");
    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 24, 20),
        '4',
        "the label clamps to the settled row"
    );

    app.update(None, Duration::from_millis(40));
    assert_eq!(app.jiggle_offset(14), Some((0, -1)), "the third beat is -y");
    assert_eq!(play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 24, 19), '4');

    app.update(None, Duration::from_millis(160));
    assert_eq!(app.jiggle_offset(14), None, "240 ms clears the wobble");
    assert_eq!(
        play_cell(&frame_buffer(&app, WIDTH, HEIGHT), 24, 20),
        '4',
        "the card is back in its slot"
    );
}

#[test]
fn an_up_merge_jiggles_down_first() {
    // 8 8 in column 0 moved Up merges at cell 0 (anchor 0,0).
    let board = Board::from_rows([[8, 0, 0, 0], [8, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    let mut app = App::new(Game::from_state(board, 0, 0, 42));
    app.update(Some(Action::Move(Direction::Up)), Duration::ZERO);
    app.update(None, Duration::from_millis(140));

    let down = frame_buffer(&app, WIDTH, HEIGHT);
    assert_eq!(
        play_cell(&down, 0, 2),
        '╭',
        "the card's top-left border is two rows down at age 0"
    );
    assert_eq!(
        play_cell(&down, 3, 4),
        '1',
        "the 16 label starts two rows down"
    );
    assert_eq!(play_cell(&down, 4, 4), '6');

    app.update(None, Duration::from_millis(40));
    let clamped = frame_buffer(&app, WIDTH, HEIGHT);
    assert_eq!(
        play_cell(&clamped, 0, 0),
        '╭',
        "at 40 ms the -y beat clamps back to row 0"
    );
    assert_eq!(play_cell(&clamped, 3, 2), '1');
    assert_eq!(play_cell(&clamped, 4, 2), '6');
}

#[test]
fn the_ripple_crosses_a_jiggling_card() {
    // Frame 12: merged cell 14 wobbles vertically (Down, +2 clamped back to row
    // 18), so its body covers the settled slot: play cells x 20..=28, y 18..=22.
    // The body is drawn in the card layer, so ripple glyphs survive on top of
    // it exactly as they do over a settled card (F2, SPEC 5.4 Layering).
    let buffer = frame_buffer(&demo_slide_app(12), WIDTH, HEIGHT);
    let crossed = (20..=28)
        .flat_map(|x| (18..=22).map(move |y| (x, y)))
        .filter(|(x, y)| is_ripple_glyph(play_cell(&buffer, *x, *y)))
        .count();

    assert!(
        crossed > 0,
        "the ripple must be drawn over the jiggling card body"
    );
}

#[test]
fn the_jiggle_ends_after_240_ms() {
    let at_176 = demo_slide_app(20);
    assert_eq!(
        at_176.jiggle_offset(14),
        Some((0, 0)),
        "age 176 has no offset"
    );
    assert_eq!(
        play_cell(&frame_buffer(&at_176, WIDTH, HEIGHT), 24, 20),
        '4'
    );

    let at_240 = demo_slide_app(24);
    assert_eq!(at_240.jiggle_offset(14), None, "age 240 clears the jiggle");
    assert_eq!(
        play_cell(&frame_buffer(&at_240, WIDTH, HEIGHT), 24, 20),
        '4',
        "the card is back in its slot"
    );
}

#[test]
fn the_jiggle_never_leaves_the_board() {
    // Cell 0 with dx = -2 clamps to x = 0.
    let board = Board::from_rows([[8, 8, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    let mut left = App::new(Game::from_state(board, 0, 0, 42));
    left.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    left.update(None, Duration::from_millis(140));
    left.update(None, Duration::from_millis(40));
    let buffer = frame_buffer(&left, WIDTH, HEIGHT);
    assert_eq!(play_cell(&buffer, 0, 0), '╭', "cell 0 clamps to column 0");
    assert_eq!(
        play_cell(&buffer, 3, 2),
        '1',
        "the label clamps with the card"
    );

    // A Right move starts with dx = -2: cell 3 moves to x = 28 at age 0.
    let board = Board::from_rows([[0, 0, 8, 8], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    let mut right = App::new(Game::from_state(board, 0, 0, 42));
    right.update(Some(Action::Move(Direction::Right)), Duration::ZERO);
    right.update(None, Duration::from_millis(140));
    let first = frame_buffer(&right, WIDTH, HEIGHT);
    assert_eq!(play_cell(&first, 28, 0), '╭', "the first beat moves left");
    assert_eq!(play_cell(&first, 31, 2), '1', "the label follows the body");

    // At 40 ms dx = +2 clamps to x = 30, so the card ends at column 38.
    right.update(None, Duration::from_millis(40));
    let buffer = frame_buffer(&right, WIDTH, HEIGHT);
    assert_eq!(play_cell(&buffer, 30, 0), '╭', "cell 3 clamps to column 30");
    assert_eq!(
        play_cell(&buffer, 38, 0),
        '╮',
        "the card stays inside 39x23"
    );
    assert_eq!(play_cell(&buffer, 33, 2), '1', "the clamped label");
}

fn merge_game_over_board() -> Board {
    Board::from_rows([
        [2, 2, 8, 16],
        [16, 32, 64, 128],
        [8, 16, 32, 64],
        [64, 128, 8, 32],
    ])
}

fn full_high_scores() -> HighScores {
    let mut table = HighScores::new();
    for (name, score) in [
        ("P0", 1000),
        ("P1", 900),
        ("P2", 800),
        ("P3", 700),
        ("P4", 600),
        ("P5", 500),
        ("P6", 400),
        ("P7", 300),
        ("P8", 200),
        ("P9", 50),
    ] {
        table.insert(name, score);
    }
    table
}

#[test]
fn the_win_overlay_waits_for_the_landing() {
    let board = Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
    let mut app = App::new(Game::from_state(board, 0, 0, 42));
    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert!(app.game.won);
    assert!(
        !frame_text(&app, WIDTH, HEIGHT).contains("2048!"),
        "the win overlay must wait for the slide"
    );

    app.update(None, Duration::from_millis(140));
    assert!(
        frame_text(&app, WIDTH, HEIGHT).contains("2048!"),
        "the win overlay appears after the landing"
    );
}

#[test]
fn the_game_over_overlay_waits_for_the_landing() {
    let mut app = App::with_table(
        Game::from_state(merge_game_over_board(), 0, 0, 42),
        full_high_scores(),
    );
    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert!(app.game.game_over);
    assert!(!app.entry_active(), "score 4 does not beat the 10th");
    assert!(
        !frame_text(&app, WIDTH, HEIGHT).contains("GAME OVER"),
        "the game-over overlay must wait for the slide"
    );

    app.update(None, Duration::from_millis(140));
    assert!(
        frame_text(&app, WIDTH, HEIGHT).contains("GAME OVER"),
        "the game-over overlay appears after the landing"
    );
}

#[test]
fn the_entry_overlay_waits_for_the_landing() {
    let mut app = App::with_table(
        Game::from_state(merge_game_over_board(), 0, 0, 42),
        HighScores::new(),
    );
    app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
    assert!(app.entry_active(), "an empty table qualifies the score");
    assert!(
        !frame_text(&app, WIDTH, HEIGHT).contains("NEW HIGH SCORE"),
        "the name-entry overlay must wait for the slide"
    );

    app.update(None, Duration::from_millis(140));
    assert!(
        frame_text(&app, WIDTH, HEIGHT).contains("NEW HIGH SCORE"),
        "the name-entry overlay appears after the landing"
    );
}
