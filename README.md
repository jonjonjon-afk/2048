# 2048 — neon terminal edition

A keyboard-first 2048 game written in Rust and played in a terminal. It keeps a
persistent best score and renders shader-style effects — a plasma background,
merge shockwaves and particle sparks — as animated 24-bit ANSI cells.

The game is specified in [`docs/SPEC.md`](docs/SPEC.md) and built with the
reviewer/coder workflow described in [`AGENTS.md`](AGENTS.md).

## Status
Intake is complete. Implementation runs through the milestones in
`docs/SPEC.md` section 8 (M0 recon, M1 core, M2 TUI, M3 effects, M4 handover).
`make run` becomes available at M1/M2.

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
