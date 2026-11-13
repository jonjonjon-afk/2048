# 2048 — neon terminal edition

A keyboard-first 2048 game written in Rust and played entirely in a terminal. It
keeps a persistent best score and renders shader-style effects — a plasma
background, merge shockwaves and particle sparks — as animated 24-bit ANSI
cells. The game is specified in [`docs/SPEC.md`](docs/SPEC.md).

## Status
- **M0 recon and plan:** approved.
- **M1 headless core + storage:** approved — exact rules, deterministic RNG, atomic high score, CLI dump.
- **M2 TUI shell + controls:** approved — layout, keys, overlays, terminal restore, snapshots.
- **M3 shader-style effects:** approved — plasma, shockwave, sparks, tile flash, spawn pop, new-best pulse.
- **M4 polish and handover:** delivered — README, clean-clone gate, full `make e2e`, acceptance evidence. Awaiting the reviewer's verdict.

The project is feature-complete for the specification. The remaining checks are
live-terminal ones that need a human at a real keyboard: truecolour and shimmer,
cursor/alternate-screen restore, and playing until BEST persists across a quit.

## Play it
```sh
make run     # the real game (release build)
make demo    # seeded 42 demo board and effects, in-memory score
```

| Key | Action |
|---|---|
| Arrow keys / WASD | move tiles |
| R | new game |
| Q / Esc | quit |
| Space / Enter | keep playing after reaching 2048 |

The minimum supported terminal is **60x30** (columns x rows); **100x36** is the
recommended size and the layout used by the committed snapshots. Below the
minimum the app shows a readable notice instead of the board and still quits
cleanly. A terminal with 24-bit colour is required for the neon palette.

## Where the high score lives
`$XDG_DATA_HOME/2048/highscore`, or `~/.local/share/2048/highscore` when
`XDG_DATA_HOME` is unset. The file holds one decimal number and a newline and is
replaced atomically. Use `--no-persist` to play without reading or writing it;
`--demo` and `--demo-merge` are also in-memory only.

## Development
```sh
make check   # cargo fmt --check, clippy with warnings as errors, all tests
make run     # play the real game
make demo    # seeded showcase board and effects
make e2e     # all integration suites + deterministic captures checked against fixtures
```

Requirements: Rust 1.88+, GNU make, and a terminal with 24-bit colour and at
least 60x30 cells (100x36 recommended). Everything is offline; there is no
network, GPU or image protocol involved.

The layout is a pure renderer over a pure core: `src/game.rs` owns the rules and
`src/rng.rs` the deterministic Xorshift64* stream, `src/highscore.rs` owns the
file format, and `src/shell.rs` is the only place that touches the terminal and
the store. `src/effects.rs` advances only from an injected `dt`, so every frame
can be reproduced with `--seed` and `--frame`.

## Project workflow
The reviewer agent plans milestones, writes requirements and audits the result;
the coder agent implements them, maintains this README, and records proof in
`docs/evidence/`. Evidence rules and the definition of done are in
[`AGENTS.md`](AGENTS.md); the latest milestone review lives in `docs/reviews/`.

## More detail
- [`docs/SPEC.md`](docs/SPEC.md) — rules, layout, CLI, milestones and acceptance.
- [`docs/EVIDENCE.md`](docs/EVIDENCE.md) — requirement and acceptance ID to proof.
- [`docs/api-notes.md`](docs/api-notes.md) — verified external facts.
- [`docs/OPEN_QUESTIONS.md`](docs/OPEN_QUESTIONS.md) — decisions and open items.
