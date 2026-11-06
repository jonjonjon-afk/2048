# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M3 (shader-style effects) — gate passed, awaiting reviewer verdict
- Last gate result: gate exit 0 (docs/evidence/M3.md "M3 gate final")
- Next action: stop at the gate and wait for the reviewer's verdict

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | gate passed, approved | approved | docs/evidence/M1.md |
| M2 | TUI shell and controls | gate passed | approved | docs/evidence/M2.md |
| M3 | Shader-style effects | gate passed | pending | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- F1: add the background-only plasma test, watch it fail, then stop the outer block painting BACKGROUND.
- F2: fill the evidence header map, assumptions, limitations and near-misses.
- F3: rebuild the M3 command log sequentially so each R8 block is well formed.
- F4: add M3 R1-R8 and A4 rows to docs/EVIDENCE.md.
- F5: set PROGRESS to "gate passed, awaiting reviewer verdict" and list the plasma/live-colour caveats.
- F6: freeze code, run the exact gate through ev.sh as "M3 gate final".
- F7: commit all M3 work; leave docs/reviews/M3.md untracked.
- F8: re-run final-HEAD R8 checks through ev.sh, append, commit again.

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
- M3: effects live in `Effects` and receive only `advance(dt)`; the shell measures and clamps live `dt` to 250 ms (Q8).
- M3: sparks use a separate fixed-seed effect RNG, so game RNG draws and spawn timing never change.
- M3: ring radius is `1 + age_ms * 3 / 100` cells, clipped to the 35x19 board block; snapshots at 64/224/384 ms grow strictly.
- M3: the plasma is a full-frame Rgb background and cards no longer paint an opaque background, so it stays dim behind the tiles.
- M3: `--demo` and `--demo-merge` both build `demo_merge_app(frame)` for render-once and start live demo play from the same pinned state.
- M3: the pinned move is seed 42 + Up; the fifth/sixth game RNG draws spawn 2 at cell 9, as reviewed.
- M3 fix: the outer frame block no longer paints `BACKGROUND` over the whole frame, so the plasma Rgb background survives to the final buffer (F1).
- M3 fix: `plasma_background_reaches_the_frame_buffer` asserts >=2 Rgb backgrounds in blank row 2 and a background change at (2,2); it failed before the F1 fix.

## Known issues
- The agent shell is headless (no tty); live rendering and truecolour are deferred to M4 (Q1).
- The `script` pty reports 0x0, so `--self-test-tui` proves the lifecycle but paints no visible cells in this environment.
- The live measured-`dt` loop compiles and is wired but was not exercised under a real tty here; the deterministic frame tests carry the animation proof.
- M3 effect snapshots are plain-text symbol captures; the 24-bit colour, the plasma shimmer and final visual acceptance remain the human/M4 check.
- M4 must confirm the fixed plasma actually shimmers behind the board in a truecolour terminal (N5).
