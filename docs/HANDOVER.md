# HANDOVER — 2048 (neon terminal edition)

> Written by the coder for M12. This is the entry point for a human or a new
> agent: how to build, play, test, where data lives, how the code is organised,
> and where the proof is. The normative behaviour is `docs/SPEC.md`; working
> rules are `AGENTS.md`.

## Current state
- All milestones M0-M11 are approved; the game is feature-complete for the SPEC.
- M12 is a documentation/repo cleanup only: the per-milestone reviews and raw
  logs were consolidated into this file, `docs/HISTORY.md` and
  `docs/evidence/ACCEPTANCE.md`. No application code, test, fixture, Makefile
  or Cargo change.
- `make check` is 147 tests, 0 failed (99 lib + 12 CLI + 28 effects + 1 restart
  + 7 score). The clean-clone gate passes.
- Platform: Linux-first, Rust 1.88+ (edition 2021), offline; no network, GPU or
  image protocol.

## Build, run, test
```sh
make run      # the real game (release build)
make demo     # seeded 42 demo board and effects, in-memory table
make check    # cargo fmt --check, clippy -D warnings, all tests
make e2e      # all integration suites + deterministic captures in docs/evidence/e2e/
```

Headless checks that need no keyboard:
```sh
cargo run --quiet -- --dump --seed 42            # exact core block, SPEC §5.6
cargo run --quiet -- --render-once --seed 42 --no-persist
cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist
script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null
```
`--self-test-tui` opens the real terminal, draws 120 frames at the live pacing,
restores the terminal, prints `self-test-tui: 120 frames in <secs>s (<fps> fps)`
and exits 0. `--demo`, `--demo-merge` and `--no-persist` always use an in-memory
store and never touch real data.

## Controls
| Key | Action |
|---|---|
| Arrow keys / WASD | move tiles |
| R | new game (keeps the high-score table) |
| Q / Esc | quit |
| Space / Enter | dismiss the win overlay and keep playing |
| H | toggle the high-score table overlay on terminals below 100 columns; after game over it switches back to the final-score view. Where the side panel is visible H does nothing. |

Name entry (`NEW HIGH SCORE` after a qualifying game over):
| Key | Action |
|---|---|
| `A-Z`, `0-9` | append one uppercase character, maximum 10 |
| Backspace | delete the last character; no-op when empty |
| Enter | save the entry and close the overlay; no-op with no characters |
| Esc | discard the entry and close the overlay |

While name entry is open, R, Q, H and the arrows do not restart, quit or toggle:
letters type into the name and arrows are ignored. A move key is ignored while
the win overlay is visible, during game over, and while the table overlay is
open during play.

## Storage
- Current table: `$XDG_DATA_HOME/2048/highscores`, or
  `~/.local/share/2048/highscores` when `XDG_DATA_HOME` is unset.
- Format: one `NAME SCORE` line per entry, at most 10, best first; names are
  1-10 uppercase `A-Z`/`0-9` characters; a saved entry stores the real game
  score, not the counted-up display.
- Best score is the first row, or 0 for an empty table. A score qualifies when
  it is > 0 and the table has fewer than 10 rows or the score beats the 10th.
- A missing or unreadable table is empty; a malformed line is skipped. Saves go
  to a sibling `.tmp` file and are renamed into place. A failed save keeps the
  game running, shows a footer warning and keeps the entry in memory.
- Legacy migration: the pre-M8 single-number file in the same directory is read
  once. When the table is missing and that file holds a positive score, it
  becomes a `--- <value>` row on the next save; the old file is never written.
- Quitting or restarting before game over saves nothing: only a name entered on
  the overlay is persisted. `--demo`, `--demo-merge`, `--no-persist`, `--dump`
  and `--render-once` use an in-memory store.

## Terminal requirements
- Minimum 60x30 (columns x rows); target and recommended 100x36. Below the
  minimum the app shows a readable notice instead of the board and still quits
  cleanly.
- 24-bit truecolour terminal (modern xterm, kitty, alacritty, foot, iTerm2).
  Effects are CPU-rendered ANSI cells: plasma background, merge ripple in
  `▀`/`▄` half blocks, particle sparks, tile flash, spawn pop, BEST pulse.
- The animation target is 60 FPS (`--fps`); core rules are instant. Raw mode and
  the alternate screen are restored on normal quit and on panic.

## Architecture map
| Area | File(s) | Owns |
|---|---|---|
| Core rules | `src/game.rs`, `src/action.rs` | slide, merge-once, score, spawn, win, game over |
| RNG | `src/rng.rs` | deterministic Xorshift64* stream, two-draw spawn order |
| Score display | `src/score.rs` | 450 ms ease-out count; display only, no clock |
| Storage | `src/highscore.rs` | `highscores` format, named table, legacy migration, file + memory stores |
| Effects | `src/effects.rs` | plasma phase, merge ripple, sparks, flash, pop, BEST pulse; advanced only by injected `dt` |
| Update | `src/update.rs`, `src/input.rs` | keys -> actions; pure `update(state, dt)`; name entry state machine |
| Shell | `src/shell.rs`, `src/terminal.rs` | event loop, deadline pacing, store wiring, terminal guard and restore |
| Renderer | `src/render.rs` | layout, cards, panel, overlays, effect painting |
| Entry points | `src/main.rs`, `src/cli.rs`, `src/demo.rs` | flag parsing, dump/render-once/self-test, demo boards |

Data flow: events -> actions -> `update(state, dt)` -> `render(state)`. `update`
does no I/O and the core has no clock; the shell supplies `dt` and owns the
store. Board scoring/merging lives only in `game`, the table format only in
`highscore`, layout only in `render`.

## Where the proof lives
- `docs/evidence/ACCEPTANCE.md` — fresh A1-A11 proof: current test names, the
  command run through `scripts/ev.sh`, the filtered result line and exit code.
- `docs/EVIDENCE.md` — acceptance ID -> test -> `ACCEPTANCE.md` label map.
- `docs/HISTORY.md` — one line per milestone M0-M11 with verdict, final commit
  and gate result.
- `docs/api-notes.md` — verified external facts with sources.
- `docs/OPEN_QUESTIONS.md` — closed decisions and their reasons.
- Git history — every deleted review and raw log is recoverable, for example
  `git show e1a86c1:docs/evidence/M7.md` or
  `git show 9f2f25e:docs/reviews/M11.md`.

## Known limitations
- The agent shell is headless: live truecolour, plasma shimmer, cursor restore
  and keyboard feel were checked by the human on a real terminal, not recorded
  here as automated output.
- The pty used for `--self-test-tui` reports 0x0; visible layout is proven by
  `TestBackend` snapshots at 100x36 and 60x30, not by the pty.
- Windows is not verified; Linux is the acceptance platform.
