# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M1 (headless core and storage) — approved
- Round: 1
- Last gate result: PASSED (exit 0; docs/evidence/M1.md "gate (Q7 resolved)")
- Next action: M2 not started; wait for the reviewer's M2 assignment

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | gate passed, approved | approved | docs/evidence/M1.md |
| M2 | TUI shell and controls | not started | - | docs/evidence/M2.md |
| M3 | Shader-style effects | not started | - | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- R1: done — package, .gitignore, Makefile and lockfile are tracked; metadata says edition 2021 / 1.88 / directories 6.
- R2: done — pure slide plus Game spawn/win/game-over; G1-G8, G10, G11 tests pass.
- R3: done — Xorshift64*, G9 cell 12, blocked-RNG, new-game and determinism tests pass.
- R4: done — FileStore/MemoryStore, missing/corrupt -> 0, sibling .tmp + rename; temp-path tests pass.
- R5: done — CLI parsing/moves tests pass; dump matches the corrected SPEC 5.6 fixture byte-for-byte.
- R6: done — make check exit 0; fmt/clippy clean; 29 lib + 5 CLI tests; no ignored/todo tests.
- R7: done — only directories in the tree; no TUI deps; protected hashes match; reviewer edits committed.
- R8: done — evidence and state updated; gate is green and the report is GATE PASSED.
- Q7: resolved by the human — SPEC corrected to cell 12; fixture and Q7 answer updated.

## Decisions (one line each, with the reason)
- Use `game2048` as both package and binary name, because R5 names `CARGO_BIN_EXE_game2048`.
- Keep dump mode on the in-memory high-score store, because the gate expects `best: 0` and dumps must never touch real data.
- Record merges as (destination cell, merged value), because R2 asks for merge count/cell/value.
- Keep `won` as the overlay flag and a separate `reached_2048` latch, because continue hides the overlay exactly once.
- Tests build temp paths from `std::env::temp_dir()`, never `HOME`, `XDG_DATA_HOME` or `default_path()`.

## Known issues
- The agent shell is headless (no tty); live rendering is deferred to M2-M4.
- `crates.io` API returned HTTP 403 in M0; M1 builds offline from the local registry cache.
