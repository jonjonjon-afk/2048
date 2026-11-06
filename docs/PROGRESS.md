# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M2 (TUI shell and controls) — gate passed pending reviewer audit
- Last gate result: M2 PASSED (exit 0; docs/evidence/M2.md "M2 gate")
- Next action: stop at the gate and wait for the reviewer's verdict

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | gate passed, approved | approved | docs/evidence/M1.md |
| M2 | TUI shell and controls | gate passed | pending | docs/evidence/M2.md |
| M3 | Shader-style effects | not started | - | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- R1: done — ratatui 0.30.2 + crossterm 0.29.0; all §5.6 flags parse; dump diff empty.
- R2: done — Action + key mapping (release ignored) + pure update(state, dt); 7 input + 5 update tests.
- R3: done — Shell over HighScoreStore; load, save on new best/quit, 3 s warning; 4 shell tests + isolation e2e.
- R4: done — 100x36 and 60x30 snapshots, 8x4 cards, too-small notice; 11 render tests.
- R5: done — header, footer legend, win and game-over overlays.
- R6: done — terminal guard, TestBackend render-once, pty self-test, no stray print macros.
- R7: done — only allowed paths, protected hashes match, no effects deps/code, no unwrap outside tests/main.
- R8: done — evidence, EVIDENCE/PROGRESS/OPEN_QUESTIONS updated; gate exit 0.

## Decisions (one line each, with the reason)
- Use `game2048` as both package and binary name, because R5 names `CARGO_BIN_EXE_game2048`.
- Keep dump mode on the in-memory high-score store, because the gate expects `best: 0` and dumps must never touch real data.
- Record merges as (destination cell, merged value), because R2 asks for merge count/cell/value.
- Keep `won` as the overlay flag and a separate `reached_2048` latch, because continue hides the overlay exactly once.
- Tests build temp paths from `std::env::temp_dir()`, never `HOME`, `XDG_DATA_HOME` or `default_path()`.
- M2: `update` returns `Effect::SaveBest` instead of touching storage, so the pure update stays I/O-free and the shell owns the store.
- M2: input module is the only crossterm user; update uses `core::time::Duration`, so the R2 purity grep stays clean.
- M2: board block is centred with floor division and the same 8x4 cards at both sizes, per the reviewer's geometry decision.
- M2: render-once and self-test always use `MemoryStore`, so previews are deterministic and isolated.
- M2: the pty self-test draws 5 frames at 16 ms; the `script` pty is 0x0, so the snapshot tests carry the visible-layout proof.

## Known issues
- The agent shell is headless (no tty); live rendering and truecolour are deferred to M4 (Q1).
- The `script` pty reports 0x0, so `--self-test-tui` proves the lifecycle but paints no visible cells in this environment.
- Live animation uses a fixed `1/fps` step; measured, clamped `dt` is M3 work (Q8).
- `--frame` is parsed but has no visible effect until M3 effects exist.
