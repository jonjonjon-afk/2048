//! Drawing: the whole UI lives here, from a 100x36 frame down to a notice.
//!
//! Rendering only reads the application state. It never mutates the game and
//! never looks at a clock, so two renders of the same state are identical.

use core::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};

use crate::anim::{clamped_anchor, Slide, CELL_PITCH_X, CELL_PITCH_Y};
use crate::effects::{Ripple, Spark, RIPPLE_BOTTOM_GLYPH, RIPPLE_TOP_GLYPH};
use crate::highscore::NAME_MAX_LEN;
use crate::update::App;

/// Width of one board card in columns.
pub const CARD_WIDTH: u16 = 9;
/// Height of one board card in rows.
pub const CARD_HEIGHT: u16 = 5;
/// Gap between two neighbouring cards in both axes.
pub const CARD_GAP: u16 = 1;
/// Total board width: four cards and three gaps.
pub const BOARD_WIDTH: u16 = CARD_WIDTH * 4 + CARD_GAP * 3;
/// Total board height: four cards and three gaps.
pub const BOARD_HEIGHT: u16 = CARD_HEIGHT * 4 + CARD_GAP * 3;
/// Width of the play-area frame: the board plus one border cell on each side.
pub const PANEL_WIDTH: u16 = BOARD_WIDTH + 2;
/// Height of the play-area frame: the board plus one border cell on each side.
pub const PANEL_HEIGHT: u16 = BOARD_HEIGHT + 2;
/// Width of the space inside a card border where the value sits.
pub const LABEL_WIDTH: u16 = CARD_WIDTH - 2;
/// Smallest supported terminal width.
pub const MIN_WIDTH: u16 = 60;
/// Smallest supported terminal height.
pub const MIN_HEIGHT: u16 = 30;
/// Smallest terminal width that shows the score panel (SPEC §4, M8).
pub const PANEL_MIN_WIDTH: u16 = 100;
/// Width of the `HIGH SCORES` panel, including its border (SPEC §4, M8).
pub const HIGH_SCORE_PANEL_WIDTH: u16 = 20;
/// Height of the `HIGH SCORES` panel, including its border (SPEC §4, M8).
pub const HIGH_SCORE_PANEL_HEIGHT: u16 = 12;
/// Width of the table overlay, including its border (SPEC §4, M8).
pub const TABLE_OVERLAY_WIDTH: u16 = 26;
/// Height of the table overlay, including its border (SPEC §4, M8).
pub const TABLE_OVERLAY_HEIGHT: u16 = 15;
/// Width of the name-entry overlay, including its border (SPEC §4, M8).
pub const ENTRY_OVERLAY_WIDTH: u16 = 34;
/// Height of the name-entry overlay, including its border (SPEC §4, M8).
pub const ENTRY_OVERLAY_HEIGHT: u16 = 7;

const BACKGROUND: Color = Color::Rgb(6, 8, 20);
const ACCENT: Color = Color::Rgb(0, 255, 200);
const MUTED: Color = Color::Rgb(110, 120, 165);
const WARNING: Color = Color::Rgb(255, 90, 90);
const SPARK_COLOUR: Color = Color::Rgb(230, 245, 255);

/// Normal header style, including the `BEST` label.
pub fn header_style() -> Style {
    Style::new()
        .fg(ACCENT)
        .bg(BACKGROUND)
        .add_modifier(Modifier::BOLD)
}

/// Header style while a new best is being celebrated.
pub fn best_pulse_style() -> Style {
    Style::new()
        .fg(Color::Rgb(255, 245, 150))
        .bg(BACKGROUND)
        .add_modifier(Modifier::BOLD)
}

/// Top-left corner of the play-area frame, or of the board inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardLayout {
    pub left: u16,
    pub top: u16,
}

/// Centre the 41x25 play-area frame in the frame (floor division).
pub fn panel_layout(width: u16, height: u16) -> BoardLayout {
    BoardLayout {
        left: width.saturating_sub(PANEL_WIDTH) / 2,
        top: height.saturating_sub(PANEL_HEIGHT) / 2,
    }
}

/// Top-left corner of the 39x23 board: the play-area frame's inner corner.
pub fn board_layout(width: u16, height: u16) -> BoardLayout {
    let panel = panel_layout(width, height);
    BoardLayout {
        left: panel.left + 1,
        top: panel.top + 1,
    }
}

/// The 20x12 `HIGH SCORES` panel beside the play area (SPEC §4, M8).
///
/// Its left edge sits two blank columns right of the play-area frame and its
/// top edge on the play area's top row. Terminals narrower than 100 columns
/// show no panel.
pub fn high_score_panel_rect(width: u16, height: u16) -> Option<Rect> {
    if width < PANEL_MIN_WIDTH || height < MIN_HEIGHT {
        return None;
    }
    let play = panel_layout(width, height);
    Some(Rect::new(
        play.left + PANEL_WIDTH + 2,
        play.top,
        HIGH_SCORE_PANEL_WIDTH,
        HIGH_SCORE_PANEL_HEIGHT,
    ))
}

/// The centred 26x15 table overlay (SPEC §4, M8).
pub fn table_overlay_rect(width: u16, height: u16) -> Rect {
    Rect::new(
        width.saturating_sub(TABLE_OVERLAY_WIDTH) / 2,
        height.saturating_sub(TABLE_OVERLAY_HEIGHT) / 2,
        TABLE_OVERLAY_WIDTH,
        TABLE_OVERLAY_HEIGHT,
    )
}

/// The centred 34x7 name-entry overlay (SPEC §4, M8).
pub fn entry_overlay_rect(width: u16, height: u16) -> Rect {
    Rect::new(
        width.saturating_sub(ENTRY_OVERLAY_WIDTH) / 2,
        height.saturating_sub(ENTRY_OVERLAY_HEIGHT) / 2,
        ENTRY_OVERLAY_WIDTH,
        ENTRY_OVERLAY_HEIGHT,
    )
}

/// Draw one complete frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    render_plasma(frame.buffer_mut(), area, app.effects.elapsed());

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    let outer = Block::new().borders(Borders::ALL);
    frame.render_widget(outer, area);

    render_header(frame, area);
    render_panel(frame, area, app);
    render_high_scores(frame, area, app);
    render_board_cards(frame, area, app);
    render_effects(frame, area, app);
    render_moving_cards(frame, area, app);
    render_board_labels(frame, area, app);
    render_footer(frame, area, app);
    render_overlay(frame, area, app);
}

/// Paint the dim plasma background over every cell of the frame.
///
/// The phase is derived only from the injected effect clock, so two renders at
/// the same elapsed time are identical and the colour ignores the wall clock.
fn render_plasma(buffer: &mut Buffer, area: Rect, elapsed: Duration) {
    let time = elapsed.as_secs_f32();
    for y in 0..area.height {
        for x in 0..area.width {
            let column = x as f32;
            let row = y as f32;
            let value = (column * 0.11 + time * 1.7).sin()
                + (row * 0.13 - time * 1.1).cos()
                + ((column + row) * 0.07 + time * 0.9).sin();
            let wave = ((value / 3.0 + 1.0) * 0.5).clamp(0.0, 1.0);
            let red = 4 + (wave * 22.0) as u8;
            let green = 6 + (wave * 30.0) as u8;
            let blue = 14 + ((1.0 - wave) * 34.0) as u8;
            let cell = &mut buffer[(area.x + x, area.y + y)];
            cell.set_symbol(" ").set_style(
                Style::new().fg(Color::Rgb(red, green, blue)).bg(Color::Rgb(
                    red / 3,
                    green / 3,
                    blue / 3,
                )),
            );
        }
    }
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from("Terminal too small"),
        Line::from(format!(
            "This game needs at least 60x30; current size is {}x{}",
            area.width, area.height
        )),
        Line::from("Please resize the window."),
    ];
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::new().fg(MUTED).bg(BACKGROUND));
    frame.render_widget(paragraph, area);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let left = "┌ 2048 ";
    let right = "┐";
    let fill = (area.width as usize).saturating_sub(left.chars().count() + right.chars().count());
    let line = format!("{left}{}{right}", "─".repeat(fill));

    let paragraph = Paragraph::new(line).style(header_style());
    frame.render_widget(paragraph, Rect::new(area.x, area.y, area.width, 1));
}

/// Draw the 41x25 play-area frame and its `SCORE`/`BEST` top edge.
///
/// The frame is a rounded block; the score line is written over its top border
/// starting two cells after the top-left corner, so both corners stay intact.
/// The rest of the edge keeps the block's `─` fill.
fn render_panel(frame: &mut Frame, area: Rect, app: &App) {
    let layout = panel_layout(area.width, area.height);
    let panel = Rect::new(layout.left, layout.top, PANEL_WIDTH, PANEL_HEIGHT);
    let border = Style::new().fg(MUTED).bg(BACKGROUND);
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    frame.render_widget(block, panel);

    let score = format!("SCORE {}", app.shown_score());
    let best = format!("BEST {}", app.game.best);
    let text = format!("{score}   {best}");
    let edge = Rect::new(layout.left + 2, layout.top, text.chars().count() as u16, 1);
    frame.buffer_mut().set_style(edge, header_style());
    for (offset, character) in text.chars().enumerate() {
        let x = layout.left + 2 + offset as u16;
        if let Some(cell) = frame.buffer_mut().cell_mut((x, layout.top)) {
            cell.set_char(character);
        }
    }

    if app.effects.is_best_pulsing() {
        let x = layout.left + 2 + (score.chars().count() + 3) as u16;
        let width = best.chars().count() as u16;
        frame
            .buffer_mut()
            .set_style(Rect::new(x, layout.top, width, 1), best_pulse_style());
    }
}

/// One table row: `{name:<10} {score:>6}` (SPEC §4, M8).
fn row_text(name: &str, score: u64) -> String {
    format!("{name:<10} {score:>6}")
}

/// Draw the wide-terminal `HIGH SCORES` panel, when it is visible.
///
/// The block is opaque (Rgb foreground and background on every cell) so the
/// plasma never shows through the table.
fn render_high_scores(frame: &mut Frame, area: Rect, app: &App) {
    let Some(rect) = high_score_panel_rect(area.width, area.height) else {
        return;
    };
    let base = Style::new().fg(MUTED).bg(BACKGROUND);
    frame.buffer_mut().set_style(rect, base);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(base)
        .style(Style::new().bg(BACKGROUND))
        .title(
            Line::from("HIGH SCORES").style(
                Style::new()
                    .fg(ACCENT)
                    .bg(BACKGROUND)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let rows: Vec<Line<'_>> = (0..HIGH_SCORE_PANEL_HEIGHT as usize - 2)
        .map(|index| match app.table.entries().get(index) {
            Some(entry) => Line::from(row_text(&entry.name, entry.score)),
            None => Line::from(""),
        })
        .collect();
    frame.render_widget(Paragraph::new(rows).style(base), inner);
}

fn cell_origin(layout: BoardLayout, cell: usize) -> Rect {
    let row = (cell / 4) as u16;
    let column = (cell % 4) as u16;
    Rect::new(
        layout.left + column * (CARD_WIDTH + CARD_GAP),
        layout.top + row * (CARD_HEIGHT + CARD_GAP),
        CARD_WIDTH,
        CARD_HEIGHT,
    )
}

fn render_board_cards(frame: &mut Frame, area: Rect, app: &App) {
    let layout = board_layout(area.width, area.height);

    for cell in 0..16 {
        // While the slide runs every slot is an empty card and the moving
        // cards are painted later, after the effect layer (SPEC 5.4, M13).
        // A jiggling merged cell draws its card at the offset in the second
        // pass below, so its slot is left alone here.
        let sliding = app.sliding();
        if !sliding && app.jiggle_dx(cell).is_some() {
            continue;
        }
        let origin = cell_origin(layout, cell);
        let value = if sliding { 0 } else { app.game.board.cell(cell) };
        let bright =
            !sliding && (app.effects.is_flashing(cell) || app.effects.is_popping(cell));
        let border = if bright {
            Style::new().fg(Color::Rgb(255, 255, 255)).bg(BACKGROUND)
        } else {
            Style::new().fg(tile_colour(value)).bg(BACKGROUND)
        };
        let card = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border);
        frame.render_widget(card, origin);
    }
}

/// Draw every moving card over the effect layer while a slide is playing.
///
/// The card is opaque (interior = renderer background), its border uses the
/// source tile's colour and its value sits on card row 2 with the same
/// centring as a settled card (SPEC 4 Moving card, M13).
fn render_moving_cards(frame: &mut Frame, area: Rect, app: &App) {
    let Some(slide) = app.slide() else {
        return;
    };
    let layout = board_layout(area.width, area.height);
    let animation = Slide { age: slide.age };

    for track in &slide.tracks {
        let (x, y) = animation.anchor(track);
        draw_tile_card(
            frame,
            layout.left as i32 + x,
            layout.top as i32 + y,
            track.value,
            false,
        );
    }
}

fn render_board_labels(frame: &mut Frame, area: Rect, app: &App) {
    // The moving cards already carry their own digits; the settled board's
    // labels appear only after the landing (SPEC 5.4, M13).
    if app.sliding() {
        return;
    }

    let layout = board_layout(area.width, area.height);

    // A jiggling merged card and its label move together, so draw the whole
    // card at the clamped offset after the effect layer (SPEC 5.4, M13).
    for cell in 0..16 {
        let value = app.game.board.cell(cell);
        if value == 0 {
            continue;
        }
        let Some(dx) = app.jiggle_dx(cell) else {
            continue;
        };
        let (x, y) = jiggle_anchor(layout, cell, dx);
        let bright = app.effects.is_flashing(cell) || app.effects.is_popping(cell);
        draw_tile_card(frame, x, y, value, bright);
    }

    for cell in 0..16 {
        let value = app.game.board.cell(cell);
        if value == 0 || app.jiggle_dx(cell).is_some() {
            continue;
        }
        let origin = cell_origin(layout, cell);
        let inner = Rect::new(origin.x + 1, origin.y + CARD_HEIGHT / 2, LABEL_WIDTH, 1);
        let bright = app.effects.is_flashing(cell) || app.effects.is_popping(cell);
        let style = if bright {
            bright_tile_style(value)
        } else {
            tile_style(value)
        };
        let label = Paragraph::new(value.to_string())
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(label, inner);
    }
}

/// The clamped absolute anchor of a jiggling card.
fn jiggle_anchor(layout: BoardLayout, cell: usize, dx: i32) -> (i32, i32) {
    let column = (cell % 4) as i32;
    let row = (cell / 4) as i32;
    let (x, y) = clamped_anchor(column * CELL_PITCH_X + dx, row * CELL_PITCH_Y);
    (layout.left as i32 + x, layout.top as i32 + y)
}

/// Draw one opaque 9x5 tile card with its value on card row 2.
fn draw_tile_card(frame: &mut Frame, x: i32, y: i32, value: u32, bright: bool) {
    let origin = Rect::new(x as u16, y as u16, CARD_WIDTH, CARD_HEIGHT);

    {
        let buffer = frame.buffer_mut();
        for row in 0..CARD_HEIGHT {
            for column in 0..CARD_WIDTH {
                if let Some(cell) = buffer.cell_mut((origin.x + column, origin.y + row)) {
                    cell.set_symbol(" ")
                        .set_style(Style::new().fg(BACKGROUND).bg(BACKGROUND));
                }
            }
        }
    }

    let border = if bright {
        Style::new().fg(Color::Rgb(255, 255, 255)).bg(BACKGROUND)
    } else {
        Style::new().fg(tile_colour(value)).bg(BACKGROUND)
    };
    let card = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    frame.render_widget(card, origin);

    let inner = Rect::new(
        origin.x + 1,
        origin.y + CARD_HEIGHT / 2,
        LABEL_WIDTH,
        1,
    );
    let style = if bright {
        bright_tile_style(value)
    } else {
        tile_style(value)
    };
    frame.render_widget(
        Paragraph::new(value.to_string())
            .alignment(Alignment::Center)
            .style(style),
        inner,
    );
}

fn render_effects(frame: &mut Frame, area: Rect, app: &App) {
    let layout = board_layout(area.width, area.height);
    let board = Rect::new(layout.left, layout.top, BOARD_WIDTH, BOARD_HEIGHT);
    let buffer = frame.buffer_mut();

    for ripple in app.effects.ripples() {
        draw_ripple(buffer, board, ripple);
    }
    for spark in app.effects.sparks() {
        draw_spark(buffer, board, spark);
    }
}

/// Draw one wave in half-cell subcells (SPEC 5.4, M11).
///
/// A lit top half is the foreground, a lit bottom half is the background of
/// the same cell, and an unlit half stays the renderer background. Both halves
/// share the `▀` glyph; a bottom-only cell uses `▄`.
fn draw_ripple(buffer: &mut Buffer, board: Rect, ripple: &Ripple) {
    let colour = tile_colour(ripple.value);

    for y in 0..BOARD_HEIGHT as i32 {
        for x in 0..BOARD_WIDTH as i32 {
            let halves = ripple.halves_at(x, y);
            let (glyph, fg, bg) = match (halves.top_lit(), halves.bottom_lit()) {
                (false, false) => continue,
                (true, false) => (
                    RIPPLE_TOP_GLYPH,
                    dimmed_colour(colour, halves.top),
                    BACKGROUND,
                ),
                (false, true) => (
                    RIPPLE_BOTTOM_GLYPH,
                    dimmed_colour(colour, halves.bottom),
                    BACKGROUND,
                ),
                (true, true) => (
                    RIPPLE_TOP_GLYPH,
                    dimmed_colour(colour, halves.top),
                    dimmed_colour(colour, halves.bottom),
                ),
            };
            if let Some(cell) = buffer.cell_mut((board.x + x as u16, board.y + y as u16)) {
                cell.set_char(glyph).set_style(Style::new().fg(fg).bg(bg));
            }
        }
    }
}

/// Dim a tile colour towards the renderer background by `amplitude` (SPEC 5.4).
///
/// `fg = bg + (colour - bg) * amplitude` per channel, rounded to the nearest
/// integer, so a crest dims as it ages and a reflection starts at 55%.
fn dimmed_colour(colour: Color, amplitude: f64) -> Color {
    let Color::Rgb(red, green, blue) = colour else {
        return colour;
    };
    let Color::Rgb(bg_red, bg_green, bg_blue) = BACKGROUND else {
        return colour;
    };
    let mix = |channel: u8, background: u8| {
        (f64::from(background) + (f64::from(channel) - f64::from(background)) * amplitude).round()
            as u8
    };

    Color::Rgb(mix(red, bg_red), mix(green, bg_green), mix(blue, bg_blue))
}

fn draw_spark(buffer: &mut Buffer, board: Rect, spark: &Spark) {
    let column = (spark.origin_cell % 4) as f32;
    let row = (spark.origin_cell / 4) as f32;
    let centre_x = column * (CARD_WIDTH + CARD_GAP) as f32 + CARD_WIDTH as f32 / 2.0;
    let centre_y = row * (CARD_HEIGHT + CARD_GAP) as f32 + CARD_HEIGHT as f32 / 2.0;
    let distance = spark.distance();
    let x = centre_x + spark.offset_x + spark.dir_x * distance;
    let y = centre_y + spark.offset_y + spark.dir_y * distance;
    if x < 0.0 || y < 0.0 {
        return;
    }
    let x = x.round() as u16;
    let y = y.round() as u16;
    if x >= BOARD_WIDTH || y >= BOARD_HEIGHT {
        return;
    }
    if let Some(cell) = buffer.cell_mut((board.x + x, board.y + y)) {
        cell.set_char(spark.symbol)
            .set_style(Style::new().fg(SPARK_COLOUR).bg(BACKGROUND));
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let legend = Rect::new(area.x + 4, area.y + area.height - 3, area.width - 8, 1);
    frame.render_widget(
        Paragraph::new("Arrows / WASD move   R restart   Q quit")
            .style(Style::new().fg(MUTED).bg(BACKGROUND)),
        legend,
    );

    if app.warning_active() {
        let warning = Rect::new(area.x + 4, area.y + area.height - 2, area.width - 8, 1);
        frame.render_widget(
            Paragraph::new("high score not saved (storage error)").style(
                Style::new()
                    .fg(WARNING)
                    .bg(BACKGROUND)
                    .add_modifier(Modifier::BOLD),
            ),
            warning,
        );
    }
}

fn render_overlay(frame: &mut Frame, area: Rect, app: &App) {
    // Win, game-over, name-entry and table overlays wait for the landing
    // (SPEC 5.4 Effect start, M13); the state behind them is already final.
    if app.sliding() {
        return;
    }
    if app.entry_active() {
        render_entry_overlay(frame, area, app);
    } else if app.game.won {
        overlay(
            frame,
            area,
            vec![
                Line::from("2048!"),
                Line::from("You reached 2048!"),
                Line::from("Space/Enter continue"),
                Line::from("R restart    Q quit"),
            ],
            ACCENT,
        );
    } else if table_overlay_visible(area, app) {
        render_table_overlay(frame, area, app);
    } else if app.game.game_over {
        overlay(
            frame,
            area,
            vec![
                Line::from("GAME OVER"),
                Line::from(format!("Score {}", app.shown_score())),
                Line::from("R restart    Q quit"),
            ],
            WARNING,
        );
    }
}

/// The table-overlay toggle rule for the draw side.
///
/// The panel replaces the overlay at 100+ columns; below that the overlay is
/// drawn whenever it is toggled on (and automatically after game over).
fn table_overlay_visible(area: Rect, app: &App) -> bool {
    area.width < PANEL_MIN_WIDTH && app.table_shown
}

/// The centred 26x15 table overlay (SPEC §4, M8).
fn render_table_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let rect = table_overlay_rect(area.width, area.height);
    let base = Style::new().fg(MUTED).bg(BACKGROUND);
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(base)
        .style(Style::new().bg(BACKGROUND))
        .title(
            Line::from("HIGH SCORES").style(
                Style::new()
                    .fg(ACCENT)
                    .bg(BACKGROUND)
                    .add_modifier(Modifier::BOLD),
            ),
        );

    let mut lines: Vec<Line<'_>> = (0..HIGH_SCORE_PANEL_HEIGHT as usize - 2)
        .map(|index| match app.table.entries().get(index) {
            Some(entry) => Line::from(row_text(&entry.name, entry.score)),
            None => Line::from(""),
        })
        .collect();
    if app.game.game_over {
        // Last two content rows: the final score and the restart/close legend.
        lines.push(Line::from(""));
        lines.push(Line::from(format!("SCORE {}", app.shown_score())));
        lines.push(Line::from("R restart   H close"));
    } else {
        // During play the bottom content row is the close legend.
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from("H close"));
    }

    render_framed(frame, rect, block, Paragraph::new(lines).style(base));
}

/// The centred 34x7 `NEW HIGH SCORE` entry overlay (SPEC §4, M8).
fn render_entry_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let rect = entry_overlay_rect(area.width, area.height);
    let accent = Style::new().fg(ACCENT).bg(BACKGROUND);
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(accent)
        .style(Style::new().bg(BACKGROUND))
        .title(
            Line::from("NEW HIGH SCORE").style(
                Style::new()
                    .fg(ACCENT)
                    .bg(BACKGROUND)
                    .add_modifier(Modifier::BOLD),
            ),
        );

    let mut name = String::from("NAME: ");
    name.push_str(app.entry_name());
    if app.entry_name().chars().count() < NAME_MAX_LEN {
        name.push('_');
    }
    let lines = vec![
        Line::from(format!("SCORE {}", app.shown_score())),
        Line::from(""),
        Line::from(name),
        Line::from(""),
        Line::from("ENTER save   ESC skip"),
    ];

    render_framed(
        frame,
        rect,
        block,
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(accent),
    );
}

/// Clear a rect, paint it opaque and draw the bordered content inside.
fn render_framed(frame: &mut Frame, rect: Rect, block: Block<'_>, content: Paragraph<'_>) {
    frame.render_widget(Clear, rect);
    frame
        .buffer_mut()
        .set_style(rect, Style::new().fg(MUTED).bg(BACKGROUND));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    frame.render_widget(content, inner);
}

fn overlay(frame: &mut Frame, area: Rect, lines: Vec<Line<'_>>, colour: Color) {
    let width = 34.min(area.width);
    let height = (lines.len() as u16 + 2).min(area.height);
    let origin = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(colour))
        .style(Style::new().bg(BACKGROUND));
    let inner = block.inner(origin);

    frame.render_widget(Clear, origin);
    frame
        .buffer_mut()
        .set_style(origin, Style::new().fg(MUTED).bg(BACKGROUND));
    frame.render_widget(block, origin);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::new().fg(colour).bg(BACKGROUND)),
        inner,
    );
}

/// The border and label colour for one tile value.
pub fn tile_colour(value: u32) -> Color {
    match value {
        0 => Color::Rgb(45, 48, 80),
        2 => Color::Rgb(0, 255, 180),
        4 => Color::Rgb(0, 200, 255),
        8 => Color::Rgb(90, 140, 255),
        16 => Color::Rgb(180, 100, 255),
        32 => Color::Rgb(255, 80, 220),
        64 => Color::Rgb(255, 60, 140),
        128 => Color::Rgb(255, 120, 60),
        256 => Color::Rgb(255, 190, 40),
        512 => Color::Rgb(220, 255, 80),
        1024 => Color::Rgb(120, 255, 120),
        _ => Color::Rgb(255, 255, 255),
    }
}

/// The unlit tile label style; the exact baseline effect tests compare against.
pub fn tile_style(value: u32) -> Style {
    Style::new()
        .fg(tile_colour(value))
        .bg(BACKGROUND)
        .add_modifier(Modifier::BOLD)
}

fn bright_tile_style(value: u32) -> Style {
    Style::new()
        .fg(Color::Rgb(255, 255, 255))
        .bg(tile_colour(value))
        .add_modifier(Modifier::BOLD)
}

/// Render the application into a ratatui buffer, for style assertions.
pub fn frame_buffer(app: &App, width: u16, height: u16) -> Buffer {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(never) => match never {},
    };
    if let Err(never) = terminal.draw(|frame| render(frame, app)) {
        match never {}
    }
    terminal.backend().buffer().clone()
}

/// Render the application to a plain-text frame of exactly `width` x `height`.
///
/// Every line is padded with the buffer's spaces, so a 100x36 frame is 36
/// lines of 100 characters and ends with one newline.
pub fn frame_text(app: &App, width: u16, height: u16) -> String {
    buffer_text(&frame_buffer(app, width, height))
}

fn buffer_text(buffer: &Buffer) -> String {
    let area = *buffer.area();
    let mut text = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let symbol = buffer.cell((x, y)).map(|cell| cell.symbol()).unwrap_or(" ");
            text.push_str(symbol);
        }
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Board, Game};

    fn seed_42_app() -> App {
        App::new(Game::new(42, 0))
    }

    fn lines(text: &str) -> Vec<Vec<char>> {
        text.lines().map(|line| line.chars().collect()).collect()
    }

    fn find_positions(grid: &[Vec<char>], needle: char) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for (y, row) in grid.iter().enumerate() {
            for (x, character) in row.iter().enumerate() {
                if *character == needle {
                    positions.push((x, y));
                }
            }
        }
        positions
    }

    #[test]
    fn render_100x36_matches_the_snapshot() {
        let text = frame_text(&seed_42_app(), 100, 36);

        assert_eq!(text.lines().count(), 36);
        assert!(text.lines().all(|line| line.chars().count() == 100));
        assert_eq!(
            text,
            include_str!("../tests/fixtures/frame_100x36_seed42.txt")
        );
    }

    #[test]
    fn render_60x30_matches_the_snapshot() {
        let text = frame_text(&seed_42_app(), 60, 30);

        assert_eq!(text.lines().count(), 30);
        assert!(text.lines().all(|line| line.chars().count() == 60));
        assert_eq!(
            text,
            include_str!("../tests/fixtures/frame_60x30_seed42.txt")
        );
    }

    #[test]
    fn card_geometry_is_9x5_with_one_cell_gaps() {
        assert_eq!(CARD_WIDTH, 9);
        assert_eq!(CARD_HEIGHT, 5);
        assert_eq!(CARD_GAP, 1);
        assert_eq!(BOARD_WIDTH, 39);
        assert_eq!(BOARD_HEIGHT, 23);
        assert_eq!(PANEL_WIDTH, 41);
        assert_eq!(PANEL_HEIGHT, 25);

        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let text = frame_text(&seed_42_app(), width, height);
            let grid = lines(&text);
            let panel = panel_layout(width, height);
            let layout = board_layout(width, height);
            let expected = if width == 100 {
                BoardLayout { left: 30, top: 6 }
            } else {
                BoardLayout { left: 10, top: 3 }
            };
            assert_eq!(layout, expected, "{width}x{height} board origin");
            assert_eq!(panel.left, layout.left - 1);
            assert_eq!(panel.top, layout.top - 1);

            let mut corners = vec![(panel.left as usize, panel.top as usize)];
            if width == PANEL_MIN_WIDTH && high_score_panel_rect(width, height).is_some() {
                // The M8 panel is a second rounded block on the same top row.
                corners.push((72, 5));
            }
            for row in 0..4u16 {
                for column in 0..4u16 {
                    let x = (layout.left + column * (CARD_WIDTH + CARD_GAP)) as usize;
                    let y = (layout.top + row * (CARD_HEIGHT + CARD_GAP)) as usize;
                    assert_eq!(grid[y][x], '╭', "{width}x{height} card {row},{column}");
                    assert_eq!(grid[y][x + 8], '╮');
                    assert_eq!(grid[y + 4][x], '╰');
                    assert_eq!(grid[y + 4][x + 8], '╯');
                    corners.push((x, y));
                }
            }

            // Cell 0's card box is (30,6)-(38,10) at 100x36.
            assert_eq!(
                find_positions(&grid, '╭'),
                corners,
                "{width}x{height} rounded corners other than the panel and 16 cards"
            );
            if width == 100 {
                assert_eq!((corners[2].0, corners[2].1), (30, 6));
                assert_eq!(grid[6][30], '╭');
                assert_eq!(grid[6][38], '╮');
                assert_eq!(grid[10][30], '╰');
                assert_eq!(grid[10][38], '╯');
            }
        }
    }

    #[test]
    fn tile_label_is_centred_on_the_middle_row() {
        // (value, expected left padding inside the 7-cell inner width)
        let cases = [(2u32, 3usize), (16, 2), (128, 2), (2048, 1)];

        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let layout = board_layout(width, height);
            let card_left = layout.left as usize;
            let card_top = layout.top as usize;
            let inner_left = card_left + 1;
            let label_row = card_top + 2;
            assert_eq!(
                (inner_left, label_row),
                if width == 100 { (31, 8) } else { (11, 5) },
                "{width}x{height} cell 0 inner box and label row"
            );

            for (value, padding) in cases {
                let mut board = Board::empty();
                board.set_cell(0, value);
                let grid = lines(&frame_text(
                    &App::new(Game::from_state(board, 0, 0, 42)),
                    width,
                    height,
                ));

                let mut digits = Vec::new();
                for (y, row) in grid
                    .iter()
                    .enumerate()
                    .skip(card_top)
                    .take(CARD_HEIGHT as usize)
                {
                    for (x, character) in row
                        .iter()
                        .enumerate()
                        .skip(card_left)
                        .take(CARD_WIDTH as usize)
                    {
                        if character.is_ascii_digit() {
                            digits.push((x, y));
                        }
                    }
                }

                let text = value.to_string();
                let expected: Vec<(usize, usize)> = (0..text.chars().count())
                    .map(|offset| (inner_left + padding + offset, label_row))
                    .collect();
                assert_eq!(
                    digits, expected,
                    "{width}x{height} value {value} must start at column {} on row {label_row} only",
                    inner_left + padding
                );
            }
        }
    }

    #[test]
    fn play_area_frame_is_41x25_and_centred() {
        for (width, height, left, top) in [(100u16, 36u16, 29usize, 5usize), (60, 30, 9, 2)] {
            let grid = lines(&frame_text(&seed_42_app(), width, height));
            let panel = panel_layout(width, height);
            assert_eq!((panel.left as usize, panel.top as usize), (left, top));
            assert_eq!(PANEL_WIDTH, 41);
            assert_eq!(PANEL_HEIGHT, 25);

            assert_eq!(grid[top][left], '╭');
            assert_eq!(grid[top][left + 40], '╮');
            assert_eq!(grid[top + 24][left], '╰');
            assert_eq!(grid[top + 24][left + 40], '╯');
            assert_eq!(
                grid[top][left..(left + PANEL_WIDTH as usize)]
                    .iter()
                    .filter(|character| **character == '╭')
                    .count(),
                1,
                "{width}x{height} play-area top edge has a second ╭"
            );
            assert_eq!(
                grid[top + 24]
                    .iter()
                    .filter(|character| **character == '╰')
                    .count(),
                1,
                "{width}x{height} bottom edge has a second ╰"
            );
        }
    }

    #[test]
    fn frame_top_edge_shows_score_and_best() {
        let game = Game::from_state(Board::empty(), 1234, 5678, 42);
        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let grid = lines(&frame_text(&App::new(game.clone()), width, height));
            let panel = panel_layout(width, height);
            let row = panel.top as usize;
            let start = panel.left as usize + 2;
            let edge: String = grid[row][start..].iter().collect();
            let text = "SCORE 1234   BEST 5678";

            assert!(
                edge.starts_with(text),
                "{width}x{height} top edge is {edge:?}"
            );
            assert_eq!(grid[row][panel.left as usize], '╭');
            assert_eq!(grid[row][start - 1], '─');
            assert_eq!(grid[row][start + text.chars().count()], '─');
            assert_eq!(grid[row][panel.left as usize + 40], '╮');
        }
    }

    #[test]
    fn header_shows_title_only() {
        let game = Game::from_state(Board::empty(), 1234, 5678, 42);
        let text = frame_text(&App::new(game), 100, 36);
        let header = text.lines().next().unwrap_or_default();

        assert!(
            header.contains("2048"),
            "header must keep the title: {header}"
        );
        assert!(
            !header.contains("SCORE"),
            "header must not show SCORE: {header}"
        );
        assert!(
            !header.contains("BEST"),
            "header must not show BEST: {header}"
        );
        assert!(header.trim_end().ends_with('┐'));
    }

    #[test]
    fn board_block_is_39x23_and_between_header_and_footer() {
        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let layout = board_layout(width, height);
            let panel = panel_layout(width, height);
            assert_eq!(BOARD_WIDTH, 39);
            assert_eq!(BOARD_HEIGHT, 23);
            assert_eq!(panel.left, if width == 100 { 29 } else { 9 });
            assert!(panel.top > 0, "the play area must start below the header");
            assert!(
                panel.top + PANEL_HEIGHT - 1 <= height - 4,
                "panel bottom {} must not reach the footer at {}",
                panel.top + PANEL_HEIGHT - 1,
                height - 3
            );
            assert_eq!(layout.top, panel.top + 1);
            assert_eq!(layout.left, panel.left + 1);
        }
    }

    #[test]
    fn seed_42_board_shows_two_tiles_at_cells_0_and_12() {
        let text = frame_text(&seed_42_app(), 100, 36);
        let grid = lines(&text);
        let layout = board_layout(100, 36);
        let mut digits = Vec::new();

        for cell in 0..16usize {
            let row = (cell / 4) as u16;
            let column = (cell % 4) as u16;
            let x = (layout.left + column * (CARD_WIDTH + CARD_GAP)) as usize;
            let y = (layout.top + row * (CARD_HEIGHT + CARD_GAP)) as usize;
            for dy in 1..CARD_HEIGHT as usize - 1 {
                for dx in 1..CARD_WIDTH as usize - 1 {
                    let character = grid[y + dy][x + dx];
                    if character.is_ascii_digit() {
                        digits.push((cell, character));
                    }
                }
            }
        }

        assert_eq!(digits, vec![(0, '2'), (12, '2')]);
    }

    #[test]
    fn too_small_terminal_shows_a_notice() {
        for (width, height) in [(59, 30), (60, 29)] {
            let text = frame_text(&seed_42_app(), width, height);
            let normalised: String = text
                .to_lowercase()
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect();

            assert!(normalised.contains("toosmall"), "{width}x{height}: {text}");
            assert!(normalised.contains("60x30"), "{width}x{height}: {text}");
            assert!(!text.contains('╭'), "{width}x{height} drew board corners");
            assert!(!text.contains('╮'));
        }
    }

    #[test]
    fn rendering_is_deterministic() {
        let app = seed_42_app();
        assert_eq!(frame_text(&app, 100, 36), frame_text(&app, 100, 36));
        assert_eq!(frame_text(&app, 60, 30), frame_text(&app, 60, 30));
    }

    #[test]
    fn footer_lists_the_controls() {
        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let text = frame_text(&seed_42_app(), width, height);
            let lines: Vec<&str> = text.lines().collect();
            let footer = lines[height as usize - 3].to_lowercase();

            for word in ["arrows", "wasd", "restart", "quit"] {
                assert!(footer.contains(word), "footer is missing {word}: {footer}");
            }
        }
    }

    #[test]
    fn win_overlay_shows_2048_continue_and_restart() {
        let mut game = Game::from_state(Board::empty(), 4096, 4096, 42);
        game.won = true;
        game.reached_2048 = true;
        let text = frame_text(&App::new(game), 100, 36).to_lowercase();

        for word in ["2048!", "continue", "restart"] {
            assert!(text.contains(word), "win overlay is missing {word}");
        }
    }

    #[test]
    fn game_over_overlay_shows_the_final_score_and_restart() {
        let board = Board::from_rows([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        let game = Game::from_state(board, 1234, 2000, 42);
        let text = frame_text(&App::new(game), 100, 36).to_lowercase();

        for word in ["game over", "1234", "restart"] {
            assert!(text.contains(word), "game-over overlay is missing {word}");
        }
    }

    fn high_scores(rows: &[(&str, u64)]) -> crate::highscore::HighScores {
        let mut table = crate::highscore::HighScores::new();
        for (name, score) in rows {
            table.insert(*name, *score);
        }
        table
    }

    fn full_high_scores() -> crate::highscore::HighScores {
        high_scores(&[
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
        ])
    }

    /// A full board with one merge available; the post-move spawn ends the game.
    fn merge_game_over_board() -> Board {
        Board::from_rows([
            [2, 2, 8, 16],
            [16, 32, 64, 128],
            [8, 16, 32, 64],
            [64, 128, 8, 32],
        ])
    }

    /// Same shape, but the move scores 16 from zero (8 + 8 at the row end).
    fn score_sixteen_game_over_board() -> Board {
        Board::from_rows([
            [16, 32, 8, 8],
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
        ])
    }

    #[test]
    fn high_score_panel_is_20x12_at_100x36() {
        let app = App::with_table(Game::new(42, 0), high_scores(&[("ALICE", 4096)]));
        let rect = high_score_panel_rect(100, 36).expect("panel at 100x36");
        assert_eq!((rect.x, rect.y, rect.width, rect.height), (72, 5, 20, 12));

        let text = frame_text(&app, 100, 36);
        let grid = lines(&text);
        assert_eq!(grid[5][72], '╭');
        assert_eq!(grid[5][91], '╮');
        assert_eq!(grid[16][72], '╰');
        assert_eq!(grid[16][91], '╯');
        let title: String = grid[5][73..84].iter().collect();
        assert_eq!(title, "HIGH SCORES");
        // SPEC §4: `{name:<10} {score:>6}` is 17 characters, 8 spaces in all.
        let expected_row = format!("{:<10} {:>6}", "ALICE", 4096);
        assert_eq!(expected_row.matches(' ').count(), 8);
        assert_eq!(expected_row.chars().count(), 17);
        let row: String = grid[6][73..90].iter().collect();
        assert_eq!(row, expected_row);
        assert_eq!(text.matches("HIGH SCORES").count(), 1);
    }

    #[test]
    fn no_panel_below_100_columns() {
        for (width, height) in [(60u16, 30u16), (99, 36)] {
            let text = frame_text(&seed_42_app(), width, height);
            assert!(
                !text.contains("HIGH SCORES"),
                "{width}x{height} drew the wide panel"
            );
            assert!(
                high_score_panel_rect(width, height).is_none(),
                "{width}x{height} still reports a panel rect"
            );
        }
    }

    #[test]
    fn table_overlay_shows_heading_rows_and_restart_at_60x30() {
        let board = Board::from_rows([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        let mut app = App::with_table(
            Game::from_state(board, 16, 0, 42),
            high_scores(&[("ALICE", 4096)]),
        );
        app.table_shown = true;

        let rect = table_overlay_rect(60, 30);
        assert_eq!((rect.x, rect.y, rect.width, rect.height), (17, 7, 26, 15));
        let text = frame_text(&app, 60, 30);
        let expected_row = format!("{:<10} {:>6}", "ALICE", 4096);
        for needle in [
            "HIGH SCORES",
            expected_row.as_str(),
            "SCORE 16",
            "R restart   H close",
        ] {
            assert!(text.contains(needle), "table overlay missing {needle:?}");
        }
        let grid = lines(&text);
        assert_eq!(grid[7][17], '╭');
        assert_eq!(grid[7][42], '╮');
        assert_eq!(grid[21][17], '╰');
        assert_eq!(grid[21][42], '╯');
    }

    #[test]
    fn table_overlay_is_shown_automatically_after_game_over_on_60x30() {
        let mut app = App::with_table(
            Game::from_state(merge_game_over_board(), 46, 0, 42),
            full_high_scores(),
        );
        let effects = app.update(
            Some(crate::action::Action::Move(crate::game::Direction::Left)),
            Duration::ZERO,
        );

        assert!(effects.is_empty());
        assert_eq!(app.game.score, 50);
        assert!(app.game.game_over);
        assert!(!app.entry_active(), "50 is not greater than the 10th");
        assert!(app.table_shown, "game over shows the table automatically");

        // Let the display count catch the real score before reading the frame.
        app.update(None, Duration::from_millis(450));
        assert_eq!(app.shown_score(), 50);

        let text = frame_text(&app, 60, 30);
        for needle in ["HIGH SCORES", "SCORE 50", "R restart   H close"] {
            assert!(
                text.contains(needle),
                "auto table overlay missing {needle:?}"
            );
        }

        app.update(Some(crate::action::Action::ToggleTable), Duration::ZERO);
        let standard = frame_text(&app, 60, 30);
        assert!(standard.contains("GAME OVER"));
        assert!(standard.contains("Score 50"));
        assert!(!standard.contains("H close"), "H must show the old overlay");

        app.update(Some(crate::action::Action::ToggleTable), Duration::ZERO);
        assert!(frame_text(&app, 60, 30).contains("R restart   H close"));
    }

    #[test]
    fn entry_overlay_shows_the_counted_score_and_the_typed_name() {
        let mut app = App::with_table(
            Game::from_state(score_sixteen_game_over_board(), 0, 0, 42),
            crate::highscore::HighScores::new(),
        );
        app.update(
            Some(crate::action::Action::Move(crate::game::Direction::Left)),
            Duration::ZERO,
        );
        assert_eq!(app.game.score, 16);
        assert!(app.game.game_over);
        assert!(app.entry_active());

        // The same `dt` that let the game over also started the count; at
        // 200 ms the G14 curve shows 13 while the real score is 16.
        app.update(None, Duration::from_millis(200));
        assert_eq!(app.shown_score(), 13);
        app.update(Some(crate::action::Action::EntryChar('a')), Duration::ZERO);
        app.update(Some(crate::action::Action::EntryChar('b')), Duration::ZERO);

        let rect = entry_overlay_rect(100, 36);
        assert_eq!((rect.x, rect.y, rect.width, rect.height), (33, 14, 34, 7));
        let text = frame_text(&app, 100, 36);
        for needle in [
            "NEW HIGH SCORE",
            "SCORE 13",
            "NAME: AB_",
            "ENTER save   ESC skip",
        ] {
            assert!(text.contains(needle), "entry overlay missing {needle:?}");
        }
        let grid = lines(&text);
        assert_eq!(grid[14][33], '╭');
        assert_eq!(grid[14][66], '╮');
        assert_eq!(grid[20][33], '╰');
        assert_eq!(grid[20][66], '╯');

        // At ten characters the underscore disappears.
        let mut full = App::with_table(
            Game::from_state(score_sixteen_game_over_board(), 0, 0, 42),
            crate::highscore::HighScores::new(),
        );
        full.update(
            Some(crate::action::Action::Move(crate::game::Direction::Left)),
            Duration::ZERO,
        );
        // M13: the name-entry overlay waits for the 140 ms landing.
        full.update(None, Duration::from_millis(140));
        for character in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'] {
            full.update(
                Some(crate::action::Action::EntryChar(character)),
                Duration::ZERO,
            );
        }
        let text = frame_text(&full, 100, 36);
        assert!(text.contains("NAME: ABCDEFGHIJ"));
        assert!(!text.contains("NAME: ABCDEFGHIJ_"));
    }
}
