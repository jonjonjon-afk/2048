# API NOTES — external facts

> Owned by the coder. Every row here is still used by the shipped code and is
> VERIFIED against a durable source (registry source path, official docs URL,
> compiler/test run, or `docs/evidence/ACCEPTANCE.md`). One-off probe rows from
> M0-M11 were dropped in M12.

| Fact | Status | Source | Used by |
|---|---|---|---|
| Toolchain: rustc 1.99.1 / cargo 1.99.1, x86_64 Linux, edition 2021; the fixed stack resolves offline as ratatui 0.30.2, crossterm 0.29.0, directories 6.0.0. | VERIFIED | `rustc -Vv`, `cargo -V`, `cargo tree --offline` (M0-M2 builds); `Cargo.lock` | Cargo.toml, every build |
| ratatui 0.30.2 re-exports `TestBackend`; `TestBackend::new(u16,u16)`, `Terminal::new`, `Terminal::draw`, `Buffer::cell`/`cell_mut`/`set_style`, `Frame::buffer_mut`/`area` exist and are not feature-gated. | VERIFIED | `ratatui-0.30.2/src/lib.rs:479-505`; `ratatui-core-0.1.2/src/backend/test.rs:32,71`; `src/buffer/buffer.rs`, `src/terminal/` | renderer, snapshots |
| `ratatui::init()` enables raw mode, enters the alternate screen and installs a panic hook; `ratatui::restore()` reverses both. crossterm 0.29 provides `event::poll/read`, `terminal::{enable_raw_mode,disable_raw_mode,size}`, and `KeyEvent`/`KeyCode`. | VERIFIED | `ratatui-0.30.2/src/init.rs:213,365,524`; `crossterm-0.29.0/src/event.rs:204,905`; `src/terminal.rs:122,218` | shell, terminal guard, input |
| `ratatui::style::Color::Rgb(u8,u8,u8)` is a tuple variant in ratatui 0.30 (crossterm's own `Rgb` is a struct variant). | VERIFIED | `ratatui-core-0.1.2/src/style/color.rs:122`; `crossterm-0.29.0/src/style/types/color.rs:87` | neon palette, effects |
| A `ratatui::Cell` has one fg and one bg colour, so a `▀` cell can carry two half colours; `▀` U+2580 and `▄` U+2584 are one code point with East_Asian_Width `A`, the same as the `╭`/`╮` corners. | VERIFIED | `ratatui-core-0.1.2/src/buffer/cell.rs`; `docs/evidence/M12.md` "R6 half-block glyph widths" | `render::draw_ripple` |
| `Paragraph` centre alignment pads with `(text_area_width / 2).saturating_sub(line_width / 2)`, so a 1-4 character tile value starts 3/2/2/1 cells into the 7-cell label rect. | VERIFIED | `ratatui-widgets-0.3.2/src/paragraph.rs:476-481`; `tile_label_is_centred_on_the_middle_row` | `render::render_board_labels` |
| ratatui `Block::new/bordered/title/style`, `Terminal::draw -> CompletedFrame` with public `area: Rect`, and `render_widget` draw deterministically into `TestBackend`. | VERIFIED | `ratatui-widgets-0.3.2/src/block.rs:366`; `ratatui-core-0.1.2/src/terminal/frame.rs:51`; snapshot tests | panel, overlays |
| directories 6.0.0: `ProjectDirs::from("", "", "2048") -> Option<ProjectDirs>`, `data_dir() -> &Path`; on Linux the path is `$XDG_DATA_HOME/2048` only when `XDG_DATA_HOME` is absolute, else `~/.local/share/2048`. | VERIFIED | `directories-6.0.0/src/lib.rs:428,473`; `src/lin.rs`; `docs/evidence/ACCEPTANCE.md` "ACC A3 restart and no-persist" | `highscore::FileStore` |
| Xorshift64* with the two-draw spawn order of SPEC §5.2 reproduces G9: seed 42 spawns cell 0 value 2 then cell 12 value 2. | VERIFIED | `rng::tests::known_seed_42_prefix_is_stable`; `game::tests::spawn_uses_the_documented_rng_order`; independent Python recomputation (M0) | `rng.rs`, `game.rs` |
| Original 2048 rules: two starting tiles, merged value added to score, no chain merge in one move, 90% 2 / 10% 4 spawn, game over when no empty cell and no equal neighbour. | VERIFIED | https://raw.githubusercontent.com/gabrielecirulli/2048/master/js/game_manager.js (fetched 2026-11-06) | `game.rs` |
| `u64::is_multiple_of` is stable since Rust 1.87.0, below the project's 1.88 MSRV. | VERIFIED | https://doc.rust-lang.org/std/primitive.u64.html#method.is_multiple_of | `game.rs` spawn draw |
| `Duration::as_millis` returns whole milliseconds; `Duration::as_secs_f64` includes the fraction; `f64::sqrt` is IEEE-correct, `f64::round` rounds half away from zero, `f64::floor` is precise, and `f64::powi` has unspecified precision (avoided). | VERIFIED | https://doc.rust-lang.org/std/time/struct.Duration.html; https://doc.rust-lang.org/std/primitive.f64.html | `effects.rs`, `render::dimmed_colour`, `score.rs` |
| `str::split_whitespace`/`lines`, `char::to_ascii_uppercase`/`is_ascii_uppercase`/`is_ascii_digit`, `Vec::insert`/`truncate`, stable `slice::sort_by_key` and `std::cmp::Reverse` behave as the table parser needs. | VERIFIED | https://doc.rust-lang.org/std/primitive.str.html; https://doc.rust-lang.org/std/primitive.char.html; https://doc.rust-lang.org/std/vec/struct.Vec.html; https://doc.rust-lang.org/std/primitive.slice.html; https://doc.rust-lang.org/std/cmp/struct.Reverse.html | `highscore.rs`, `update.rs` |
| `Instant + Duration`, `later - earlier` and `saturating_duration_since` are stable; a negative difference returns zero instead of panicking. | VERIFIED | `shell::tests::frame_pacing_skips_missed_frames`; https://doc.rust-lang.org/std/time/struct.Instant.html | `shell::next_frame_deadline`, live loop |
| On a headless pty (`script -qec`, `TERM` unset) crossterm reports 0x0, but `ratatui::init`, five 0x0 draws, the 120-frame self-test and `restore` still exit 0 and print the measured line. | VERIFIED | `docs/evidence/ACCEPTANCE.md` "ACC A5 restore and pty self-test", "ACC A8 60 FPS pacing" | `--self-test-tui`, terminal guard |

Caveat outside the table: whether the terminal emulator draws 24-bit colour and
the half-block glyphs in its own font is a terminal property, not a crate
contract. The human accepted the live look on 2026-11-13; `docs/HANDOVER.md`
records that the agent shell cannot see it.
