# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M1 (headless core and storage) — BLOCKED on a SPEC conflict (Q7)
- Round: 1
- Last gate result: FAILED (make check stops on the dump fixture; see docs/evidence/M1.md "gate command")
- Next action: wait for the reviewer/human decision on Q7; do not start M2

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | blocked: Q7 SPEC conflict | - | docs/evidence/M1.md |
| M2 | TUI shell and controls | not started | - | docs/evidence/M2.md |
| M3 | Shader-style effects | not started | - | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- R1: done — package, .gitignore, Makefile and lockfile are tracked; metadata says edition 2021 / 1.88 / directories 6.
- R2: done — pure slide plus Game spawn/win/game-over; G1-G8, G10, G11 tests pass.
- R3: done — Xorshift64*, G9 cell 12, blocked-RNG, new-game and determinism tests pass.
- R4: done — FileStore/MemoryStore, missing/corrupt -> 0, sibling .tmp + rename; temp-path tests pass.
- R5: blocked — CLI parsing and moves tests pass; the SPEC 5.6 dump block disagrees with G9 (Q7).
- R6: blocked — fmt and clippy pass; make check stops on the conflicting dump assertions.
- R7: done — only directories in the tree; no TUI deps; protected hashes match; reviewer edits committed.
- R8: blocked — evidence and state updated, but STATUS is BLOCKED, not GATE PASSED.
- Q7: proposed default is to follow §5.2/G9 (cell 12) and correct the §5.6 example and M1 fixture.

## Decisions (one line each, with the reason)
- Use `game2048` as both package and binary name, because R5 names `CARGO_BIN_EXE_game2048`.
- Keep dump mode on the in-memory high-score store, because the gate expects `best: 0` and dumps must never touch real data.
- Record merges as (destination cell, merged value), because R2 asks for merge count/cell/value.
- Keep `won` as the overlay flag and a separate `reached_2048` latch, because continue hides the overlay exactly once.
- Tests build temp paths from `std::env::temp_dir()`, never `HOME`, `XDG_DATA_HOME` or `default_path()`.

## Known issues
- The agent shell is headless (no tty); live rendering is deferred to M2-M4.
- `crates.io` API returned HTTP 403 in M0; M1 builds offline from the local registry cache.
