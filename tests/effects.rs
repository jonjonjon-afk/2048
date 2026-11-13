//! M7 effect tests: injected clock, plasma, merge ripple, sparks, flashes and pulse.
//!
//! Every animation frame is rendered from fixed state and fixed `dt`; nothing
//! here reads the wall clock or the real high-score file.

use std::collections::HashSet;
use std::time::Duration;

use game2048::demo::demo_merge_app;
use game2048::effects::{
    Effects, BEST_PULSE_MILLIS, CELL_CENTRE_X, CELL_CENTRE_Y, CELL_PITCH_X, CELL_PITCH_Y,
    FLASH_MILLIS, POP_MILLIS, RIPPLE_FRONT_WINDOW, RIPPLE_ROW_ASPECT, RIPPLE_SYMBOLS,
    RIPPLE_WALL_FACTOR, SPARKS_PER_MERGE, SPARK_MILLIS, SPARK_SYMBOLS,
};
use game2048::game::{Board, Direction, Game, Merge};
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

/// Every ripple glyph inside the 39x23 play area, with its play-area cell.
fn ripple_cells(buffer: &Buffer) -> Vec<(i32, i32, char)> {
    let mut cells = Vec::new();
    for y in 0..PLAY_HEIGHT {
        for x in 0..PLAY_WIDTH {
            let glyph = play_cell(buffer, x, y);
            if RIPPLE_SYMBOLS.contains(&glyph) {
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

#[test]
fn ripple_front_at_100ms_matches_the_round_hand_table() {
    let buffer = frame_buffer(&app_at(100), WIDTH, HEIGHT);

    assert_eq!(play_cell(&buffer, 0, 2), '▓', "(0,2) is the direct front");
    assert_eq!(play_cell(&buffer, 9, 2), '▓', "(9,2) is the direct front");
    assert_eq!(play_cell(&buffer, 7, 1), '▓', "(7,1) is on the round front");
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 0, 4)),
        "(0,4) is off the round front (a square would light it)"
    );
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 4, 2)),
        "(4,2) is inside the ring, not on it"
    );
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 5, 5)),
        "(5,5) is inside the ring, not on it"
    );
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 20, 12)),
        "(20,12) is outside the ring"
    );

    // The front is a ring: the centre cell is not painted by the wave alone.
    let app = app_at(100);
    let ripple = app.effects.ripples()[0];
    assert!(ripple.glyph_at(7, 1).is_some());
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
    assert_eq!(play_cell(&buffer, 7, 1), '▓', "100 ms (7,1) in the frame");
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 0, 4)),
        "100 ms (0,4) is blank in the frame"
    );

    // 150 ms: the direct front reaches (0,0) at d=6.727.
    let at_150 = app_at(150);
    assert_eq!(
        at_150.effects.ripples()[0].glyph_at(0, 0),
        Some('▓'),
        "150 ms (0,0)"
    );
    assert_eq!(
        play_cell(&frame_buffer(&at_150, WIDTH, HEIGHT), 0, 0),
        '▓',
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
    assert_eq!(play_cell(&buffer, 13, 2), '▓', "224 ms (13,2) in the frame");
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&buffer, 13, 10)),
        "224 ms (13,10) is blank in the frame"
    );
}

#[test]
fn ripple_reflects_off_the_left_wall() {
    let at_150 = frame_buffer(&app_at(150), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_150, 10, 2), '▓', "(10,2) direct at 150 ms");
    assert_eq!(
        play_cell(&at_150, 0, 2),
        '▒',
        "(0,2) is the left-wall image at 150 ms"
    );

    let at_224 = frame_buffer(&app_at(224), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_224, 13, 2), '▓', "(13,2) direct at 224 ms");
    // R3 pins the reflected crest at (3,2); at 224 ms a spark's glyph is drawn
    // over that cell, so this buffer pin cannot pass without changing sparks or
    // the renderer. See docs/OPEN_QUESTIONS.md Q14.
    assert_eq!(
        play_cell(&at_224, 3, 2),
        '▒',
        "(3,2) is the left-wall image at 224 ms"
    );
    assert!(
        !RIPPLE_SYMBOLS.contains(&play_cell(&at_224, 9, 2)),
        "(9,2) sits in the gap between the direct and reflected crests"
    );
}

#[test]
fn ripple_fades_to_sta_and_expires_at_900ms() {
    let at_480 = frame_buffer(&app_at(480), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_480, 23, 2), '▒', "(23,2) at 480 ms");
    assert_eq!(
        play_cell(&at_480, 13, 2),
        '░',
        "(13,2) is the weak left-wall image at 480 ms"
    );

    let at_700 = frame_buffer(&app_at(700), WIDTH, HEIGHT);
    assert_eq!(play_cell(&at_700, 32, 2), '░', "(32,2) at 700 ms");
    let strong: Vec<(i32, i32, char)> = ripple_cells(&at_700)
        .into_iter()
        .filter(|(_, _, glyph)| *glyph == '▓' || *glyph == '▒')
        .collect();
    assert!(
        strong.is_empty(),
        "700 ms must show no ▓ or ▒ anywhere, found {strong:?}"
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
    assert_eq!(play_cell(&buffer, 13, 2), '▓', "frame 14 ripple at (13,2)");
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
    let panel = panel_layout(WIDTH, HEIGHT);
    let colour_at = |millis: u64, x: i32, y: i32| -> Color {
        let buffer = frame_buffer(&app_at(millis), WIDTH, HEIGHT);
        buffer[(panel.left + 1 + x as u16, panel.top + 1 + y as u16)].fg
    };

    // SPEC 5.4 Brightness: fg = bg + (colour - bg) * amplitude, rounded, with
    // bg = Rgb(6, 8, 20) and the value-4 colour Rgb(0, 200, 255).
    let direct_224 = colour_at(224, 13, 2);
    assert_eq!(direct_224, Color::Rgb(1, 152, 197), "224 ms direct crest");
    // (3,2) is the label row of the value-4 card, so the label layer replaces
    // the ripple foreground (Q13); the R4 literal cannot be read there. Q14.
    assert_eq!(
        colour_at(224, 3, 2),
        Color::Rgb(4, 87, 117),
        "224 ms left-wall crest"
    );
    assert_eq!(
        colour_at(480, 23, 2),
        Color::Rgb(3, 98, 130),
        "480 ms direct crest"
    );
    assert_eq!(
        colour_at(700, 32, 2),
        Color::Rgb(5, 51, 72),
        "700 ms direct crest"
    );

    let reflected_224 = color_luma(colour_at(224, 3, 2));
    assert!(
        color_luma(direct_224) > reflected_224,
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
