# 2048 — neon terminal edition

A keyboard-first 2048 game written in Rust and played entirely in a terminal. It
keeps a persistent high-score table and renders shader-style effects — a plasma
background, a merge ripple and particle sparks — as animated 24-bit ANSI cells.
The game is specified in [`docs/SPEC.md`](docs/SPEC.md).

## Status
- **M0 recon and plan:** approved.
- **M1 headless core + storage:** approved — exact rules, deterministic RNG, atomic high-score table, CLI dump.
- **M2 TUI shell + controls:** approved — layout, keys, overlays, terminal restore, snapshots.
- **M3 shader-style effects:** approved — plasma, sparks, tile flash, spawn pop, new-best pulse.
- **M4 polish and handover:** approved — README, clean-clone gate, full `make e2e`, acceptance evidence.
- **M5 prettier board and real 60 FPS:** approved — 9x5 cards, `SCORE`/`BEST` on the play-area edge, deadline pacing.
- **M6 score count-up:** approved — 450 ms ease-out display that never feeds back into the rules.
- **M7 merge ripple:** approved — the bouncing, fading 900 ms wave.
- **M8 arcade high-score table:** approved — named table, name entry, side panel or `H` overlay, legacy migration.
- **M9 handover refresh:** approved — current README, acceptance rows, clean-clone gate.
- **M10 round merge ripple:** approved — aspect-corrected Euclidean wavefront.
- **M11 half-block ripple:** approved — `▀`/`▄` subcells with two colours per cell.
- **M12 lean handover:** approved — docs consolidated, no application code change.
- **M13 tile slide and fusion jiggle:** approved — a 140 ms ease-out glide, move keys ignored until it lands, effects starting at the landing, and a 240 ms merge jiggle.
- **M14 jiggle along the merge axis:** approved — columns for left/right moves, rows for up/down moves.

The project is feature-complete for the specification. The remaining checks are
live-terminal ones that need a human at a real keyboard: truecolour and shimmer,
cursor/alternate-screen restore, and playing until a named score persists across
a quit and relaunch.

## Play it
```sh
make run     # the real game (release build)
make demo    # seeded 42 demo board and effects, in-memory table
```

| Key | Action |
|---|---|
| Arrow keys / WASD | move tiles |
| R | new game |
| Q / Esc | quit |
| Space / Enter | keep playing after reaching 2048 |
| H | toggle the high-score table on terminals below 100 columns; after game over it switches back to the final-score view. Where the side panel is visible `H` does nothing. |

When a game ends with a qualifying score, a centred **NEW HIGH SCORE** overlay
replaces the game-over view. Its keys are separate from the game controls:

| Key | Action |
|---|---|
| `A-Z`, `0-9` | append one uppercase character (maximum 10) |
| Backspace | delete the last character; no-op when empty |
| Enter | save the entry and close the overlay; does nothing with no characters |
| Esc | discard the entry and close the overlay |

While name entry is open, `R`, `Q`, `H` and the arrow keys do not restart, quit
or toggle anything: letters are typed into the name and arrows are ignored.

A move key pressed while a tile is still gliding is ignored: each move's tiles
slide for 140 ms on an ease-out curve, the merge and spawn effects start when
they land, and a merged card then wobbles for 240 ms along the merge axis
(columns after a left/right move, rows after an up/down move). `R` clears it.

The minimum supported terminal is **60x30** (columns x rows); **100x36** is the
recommended size and the layout used by the committed snapshots. Below the
minimum the app shows a readable notice instead of the board and still quits
cleanly. A terminal with 24-bit colour is required for the neon palette.

## Where the high scores live
`$XDG_DATA_HOME/2048/highscores`, or `~/.local/share/2048/highscores` when
`XDG_DATA_HOME` is unset. The table is plain text: one `NAME SCORE` line per
entry, at most 10 entries, sorted by score with the best on the first row. Names
are 1-10 uppercase `A-Z`/`0-9` characters. A missing or unreadable file is an
empty table; a malformed line is skipped. Saves go to a sibling `.tmp` file and
are renamed into place.

The pre-M8 file (the singular form of `highscores`, in the same directory) held
one decimal number. It is read once for migration: when the table is missing and
that file holds a positive score, the score becomes a `--- <v>` row on the next
save, and the old file is never written again.

`--no-persist`, `--demo` and `--demo-merge` use an in-memory table and never
touch either file. Quitting or restarting before game over saves nothing: only a
name entered on the NEW HIGH SCORE overlay is stored, using the real game score
rather than the counted-up display.

## Development
```sh
make check   # cargo fmt --check, clippy with warnings as errors, all tests
make run     # play the real game
make demo    # seeded showcase board and effects
make e2e     # all integration suites + deterministic captures in docs/evidence/e2e/, checked against fixtures
```

Headless captures (no keyboard needed):
```sh
cargo run --quiet -- --render-once --demo-slide --frame 9 --no-persist                       # 100x36 motion frame
cargo run --quiet -- --render-once --demo-slide --frame 9 --size 60x30 --no-persist         # the 60x30 capture
```
`--size <WxH>` sets the `--render-once` frame size (default `100x36`, minimum
`60x30`); on its own it is an error. It is the documented way to regenerate the
60x30 capture.

`make check` runs 190 tests: 124 lib + 17 CLI + 41 effects + 1 restart +
7 score. The committed snapshots and fixtures are the exact expected frames.

Requirements: Rust 1.88+, GNU make, and a terminal with 24-bit colour and at
least 60x30 cells (100x36 recommended). Everything is offline; there is no
network, GPU or image protocol involved.

The layout is a pure renderer over a pure core: `src/game.rs` owns the rules and
`src/rng.rs` the deterministic Xorshift64* stream, a dedicated storage module
owns the table format, `src/anim.rs` owns the slide and jiggle geometry over
injected ages, and `src/shell.rs` is the only place that touches the terminal
and the store. `src/effects.rs` advances only from an injected `dt`, so every
frame can be reproduced with `--seed` and `--frame`.

## Project workflow
The reviewer agent plans milestones, writes requirements and audits the result;
the coder agent implements them, maintains this README, and records proof in
`docs/evidence/`. Evidence rules and the definition of done are in
[`AGENTS.md`](AGENTS.md); the latest milestone review lives in `docs/reviews/`.

## More detail
- [`docs/HANDOVER.md`](docs/HANDOVER.md) — entry point: run commands, controls, storage, architecture, proof.
- [`docs/HISTORY.md`](docs/HISTORY.md) — one line per milestone M0-M14 with verdict and commit.
- [`docs/SPEC.md`](docs/SPEC.md) — rules, layout, CLI, milestones and acceptance.
- [`docs/EVIDENCE.md`](docs/EVIDENCE.md) — acceptance ID to proof in `docs/evidence/ACCEPTANCE.md`.
- [`docs/api-notes.md`](docs/api-notes.md) — verified external facts.
- [`docs/OPEN_QUESTIONS.md`](docs/OPEN_QUESTIONS.md) — closed decisions and reasons.
