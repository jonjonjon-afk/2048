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

use crate::effects::{Shockwave, Spark, RING_SYMBOLS};
use crate::update::App;

/// Width of one board card in columns.
pub const CARD_WIDTH: u16 = 8;
/// Height of one board card in rows.
pub const CARD_HEIGHT: u16 = 4;
/// Gap between two neighbouring cards in both axes.
pub const CARD_GAP: u16 = 1;
/// Total board width: four cards and three gaps.
pub const BOARD_WIDTH: u16 = CARD_WIDTH * 4 + CARD_GAP * 3;
/// Total board height: four cards and three gaps.
pub const BOARD_HEIGHT: u16 = CARD_HEIGHT * 4 + CARD_GAP * 3;
/// Smallest supported terminal width.
pub const MIN_WIDTH: u16 = 60;
/// Smallest supported terminal height.
pub const MIN_HEIGHT: u16 = 30;

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

/// Where the board block starts inside a frame of the given size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardLayout {
    pub left: u16,
    pub top: u16,
}

/// Centre the 35x19 board block horizontally and vertically (floor division).
pub fn board_layout(width: u16, height: u16) -> BoardLayout {
    BoardLayout {
        left: width.saturating_sub(BOARD_WIDTH) / 2,
        top: height.saturating_sub(BOARD_HEIGHT) / 2,
    }
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

    render_header(frame, area, app);
    render_board_cards(frame, area, app);
    render_effects(frame, area, app);
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

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let left = "┌ 2048 ";
    let score = format!(" SCORE {}   ", app.game.score);
    let best = format!("BEST {} ┐", app.game.best);
    let fill = (area.width as usize)
        .saturating_sub(left.chars().count() + score.chars().count() + best.chars().count());
    let line = format!("{left}{}{score}{best}", "─".repeat(fill));

    let paragraph = Paragraph::new(line).style(header_style());
    frame.render_widget(paragraph, Rect::new(area.x, area.y, area.width, 1));

    if app.effects.is_best_pulsing() {
        let x = area.x + (left.chars().count() + fill + score.chars().count()) as u16;
        let width = format!("BEST {}", app.game.best).chars().count() as u16;
        frame
            .buffer_mut()
            .set_style(Rect::new(x, area.y, width, 1), best_pulse_style());
    }
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
        let origin = cell_origin(layout, cell);
        let value = app.game.board.cell(cell);
        let bright = app.effects.is_flashing(cell) || app.effects.is_popping(cell);
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

fn render_board_labels(frame: &mut Frame, area: Rect, app: &App) {
    let layout = board_layout(area.width, area.height);

    for cell in 0..16 {
        let value = app.game.board.cell(cell);
        if value == 0 {
            continue;
        }
        let origin = cell_origin(layout, cell);
        let inner = Rect::new(
            origin.x + 1,
            origin.y + 1,
            origin.width - 2,
            origin.height - 2,
        );
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

fn render_effects(frame: &mut Frame, area: Rect, app: &App) {
    let layout = board_layout(area.width, area.height);
    let board = Rect::new(layout.left, layout.top, BOARD_WIDTH, BOARD_HEIGHT);
    let buffer = frame.buffer_mut();

    for wave in app.effects.shockwaves() {
        draw_shockwave(buffer, board, wave);
    }
    for spark in app.effects.sparks() {
        draw_spark(buffer, board, spark);
    }
}

fn draw_shockwave(buffer: &mut Buffer, board: Rect, wave: &Shockwave) {
    let radius = wave.radius();
    let column = (wave.cell % 4) as i32;
    let row = (wave.cell / 4) as i32;
    let centre_x = column * (CARD_WIDTH as i32 + CARD_GAP as i32) + CARD_WIDTH as i32 / 2;
    let centre_y = row * (CARD_HEIGHT as i32 + CARD_GAP as i32) + CARD_HEIGHT as i32 / 2;
    let colour = tile_colour(wave.value);

    for y in 0..BOARD_HEIGHT as i32 {
        for x in 0..BOARD_WIDTH as i32 {
            let distance = (x - centre_x).abs().max((y - centre_y).abs());
            if distance == 0 || distance > radius || distance + 2 < radius {
                continue;
            }
            let index = (distance + 2 - radius).clamp(0, 2) as usize;
            if let Some(cell) = buffer.cell_mut((board.x + x as u16, board.y + y as u16)) {
                cell.set_char(RING_SYMBOLS[index])
                    .set_style(Style::new().fg(colour).bg(BACKGROUND));
            }
        }
    }
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
    if app.game.won {
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
    } else if app.game.game_over {
        overlay(
            frame,
            area,
            vec![
                Line::from("GAME OVER"),
                Line::from(format!("Score {}", app.game.score)),
                Line::from("R restart    Q quit"),
            ],
            WARNING,
        );
    }
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
    fn board_cards_are_8x4_with_one_cell_gaps() {
        for (width, height) in [(100, 36), (60, 30)] {
            let text = frame_text(&seed_42_app(), width, height);
            let grid = lines(&text);
            let layout = board_layout(width, height);
            let mut corners = Vec::new();

            for row in 0..4u16 {
                for column in 0..4u16 {
                    let x = (layout.left + column * (CARD_WIDTH + CARD_GAP)) as usize;
                    let y = (layout.top + row * (CARD_HEIGHT + CARD_GAP)) as usize;
                    assert_eq!(grid[y][x], '╭', "{width}x{height} card {row},{column}");
                    assert_eq!(grid[y][x + 7], '╮');
                    assert_eq!(grid[y + 3][x], '╰');
                    assert_eq!(grid[y + 3][x + 7], '╯');
                    corners.push((x, y));
                }
            }

            assert_eq!(find_positions(&grid, '╭'), corners);
            assert_eq!(layout.left, if width == 100 { 32 } else { 12 });
            assert_eq!(
                layout.left + BOARD_WIDTH,
                corners.last().unwrap().0 as u16 + 8
            );
        }
    }

    #[test]
    fn board_block_is_35x19_and_between_header_and_footer() {
        for (width, height) in [(100u16, 36u16), (60, 30)] {
            let layout = board_layout(width, height);
            assert_eq!(BOARD_WIDTH, 35);
            assert_eq!(BOARD_HEIGHT, 19);
            assert_eq!(layout.left, if width == 100 { 32 } else { 12 });
            assert!(layout.top > 0, "board must start below the header");
            assert!(
                layout.top + BOARD_HEIGHT < height - 3,
                "board must end above the footer"
            );
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
            for dy in 1..3usize {
                for dx in 1..7usize {
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
    fn header_shows_title_score_and_best() {
        let game = Game::from_state(Board::empty(), 1234, 5678, 42);
        let text = frame_text(&App::new(game), 100, 36);
        let header = text.lines().next().unwrap_or_default();
        let normalised = header.split_whitespace().collect::<Vec<_>>().join(" ");

        let mut cursor = 0;
        for token in ["2048", "SCORE", "1234", "BEST", "5678"] {
            let Some(found) = normalised[cursor..].find(token) else {
                panic!("header is missing {token}: {normalised}");
            };
            cursor += found + token.len();
        }
    }

    #[test]
    fn footer_lists_the_controls() {
        let text = frame_text(&seed_42_app(), 100, 36);
        let footer = text
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        for word in ["arrows", "wasd", "restart", "quit"] {
            assert!(footer.contains(word), "footer is missing {word}: {footer}");
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
}
