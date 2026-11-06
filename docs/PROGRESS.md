# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M1 (headless core and storage) — in progress
- Round: 1
- Last gate result: not run yet
- Next action: implement R1-R8, run the gate, stop for review

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | in progress | - | docs/evidence/M1.md |
| M2 | TUI shell and controls | not started | - | docs/evidence/M2.md |
| M3 | Shader-style effects | not started | - | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- R1: Cargo package game2048 (lib + bin, edition 2021, rust-version 1.88, directories 6), .gitignore, Makefile, lockfile.
- R2: game module with 4x4 slide, merge-once, score, spawn, win/continue, game over; golden tests G1-G8, G10, G11.
- R3: Xorshift64* plus the two-draw spawn order; G9, blocked-RNG, new-game and determinism tests.
- R4: highscore trait with FileStore (sibling .tmp then rename) and MemoryStore; missing/corrupt -> 0; temp-path tests.
- R5: CLI --seed, --moves, --dump, --help; SPEC 5.6 fixture; integration and parsing tests.
- R6: make check green, clippy -D warnings clean, more than 12 tests.
- R7: no TUI dependencies, no protected-file edits, verify hashes.
- R8: fill docs/evidence/M1.md, docs/EVIDENCE.md, PROGRESS and OPEN_QUESTIONS, then report one line per R1-R8.

## Decisions (one line each, with the reason)
- Use `game2048` as both package and binary name, because R5 names `CARGO_BIN_EXE_game2048`.
- Keep dump mode on the in-memory high-score store, because the gate expects `best: 0` and dumps must never touch real data.
- Record merges as (destination cell, merged value), because R2 asks for merge count/cell/value.
- Keep `won` as the overlay flag and a separate `reached_2048` latch, because continue hides the overlay exactly once.
- Tests build temp paths from `std::env::temp_dir()`, never `HOME`, `XDG_DATA_HOME` or `default_path()`.

## Known issues
- The agent shell is headless (no tty); live rendering is deferred to M2-M4.
- `crates.io` API returned HTTP 403 in M0; M1 builds offline from the local registry cache.
