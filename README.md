# 2048 — neon terminal edition

A keyboard-first 2048 game written in Rust and played in a terminal. It keeps a
persistent best score and renders shader-style effects — a plasma background,
merge shockwaves and particle sparks — as animated 24-bit ANSI cells.

The game is specified in [`docs/SPEC.md`](docs/SPEC.md) and built with the
reviewer/coder workflow described in [`AGENTS.md`](AGENTS.md).

## Status
- **M0 recon and plan:** approved.
- **M1 headless core + storage:** approved — exact rules, deterministic RNG, atomic high score, CLI dump; 34 tests green.
- **Next:** M2 TUI shell and controls (layout, keys, overlays, terminal restore, snapshots). `make run` is still a usage stub until M2.

`make check` works today. `make demo` and the full `make e2e` become real in M2/M3.

## Controls
| Key | Action |
|---|---|
| Arrow keys / WASD | move tiles |
| R | new game |
| Q / Esc | quit |
| Space / Enter | keep playing after reaching 2048 |

## What makes it different
- **Shader-style effects:** the same maths a GPU shader would use (plasma,
  radial waves, particle motion) is evaluated per terminal cell and drawn with
  truecolour styles. No GPU or graphics protocol is required.
- **Deterministic:** every board and effect frame can be reproduced from
  `--seed` and `--frame`, so the screenshots in `docs/evidence/` are testable.
- **Safe storage:** the best score lives in the per-user data directory; demo
  and test modes never touch it.

## Development
```sh
make check   # format, clippy with warnings as errors, all tests
make run     # play
make demo    # seeded showcase board and effects
make e2e     # deterministic captures + persistence round-trip
```

Requirements: Rust 1.88+, GNU make, and a terminal with 24-bit colour and at
least 60x30 cells (100x36 recommended).

## Where the high score lives
`$XDG_DATA_HOME/2048/highscore`, or `~/.local/share/2048/highscore` when
`XDG_DATA_HOME` is unset. Use `--no-persist` to play without writing it.

## Project workflow
The reviewer agent owns this README, `AGENTS.md`, `docs/SPEC.md` and the
milestone reviews. The coder subagent writes application code and proof.
Evidence rules and the definition of done are in `AGENTS.md`; the audit
checklist is in `docs/REVIEW_PLAYBOOK.md`.
