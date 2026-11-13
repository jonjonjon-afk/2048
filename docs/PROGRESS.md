# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M4 (polish and handover) — approved with follow-ups; U1/U2 done, awaiting reviewer confirmation
- Last gate result: gate exit 0 (docs/evidence/M4.md "M4 gate final round 2", run at 641a5b1 after the F2 revert)
- Next action: stop; the human still owes the live-terminal checks

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | gate passed, approved | approved | docs/evidence/M1.md |
| M2 | TUI shell and controls | gate passed | approved | docs/evidence/M2.md |
| M3 | Shader-style effects | gate passed | approved with follow-ups | docs/evidence/M3.md |
| M4 | Polish and handover | gate passed | pending | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- U1: point the R6 and A6 proof labels in `docs/EVIDENCE.md` at `M4 gate final round 2`.
- U1: record the grep proof through ev.sh; no other EVIDENCE row changes.
- U2: commit `docs/reviews/M4.md` byte-unchanged (no content edit).
- U2: record the committed-vs-working hash and the clean tree through ev.sh.
- Commit the U1 evidence append after the U2 check; no code, test, Makefile or README change.

## Decisions (one line each, with the reason)
- Use `game2048` as both package and binary name, because R5 in M1 names `CARGO_BIN_EXE_game2048`.
- Keep dump mode on the in-memory high-score store, because the gate expects `best: 0` and dumps must never touch real data.
- Record merges as (destination cell, merged value), because M1 R2 asks for merge count/cell/value.
- Keep `won` as the overlay flag and a separate `reached_2048` latch, because continue hides the overlay exactly once.
- Tests build temp paths from `std::env::temp_dir()`, never `HOME`, `XDG_DATA_HOME` or `default_path()`.
- M2: `update` returns `Effect::SaveBest` instead of touching storage, so the pure update stays I/O-free and the shell owns the store.
- M2: input module is the only crossterm user; update uses `core::time::Duration`, so the purity grep stays clean.
- M2: board block is centred with floor division and the same 8x4 cards at both sizes, per the reviewer's geometry decision.
- M2: render-once and self-test always use `MemoryStore`, so previews are deterministic and isolated.
- M3: effects live in `Effects` and receive only `advance(dt)`; the shell measures and clamps live `dt` to 250 ms (Q8).
- M3: ring radius is `1 + age_ms * 3 / 100` cells, clipped to the 35x19 board block; radii at frames 4/14/24 are 2/7/12.
- M3: the plasma is a full-frame Rgb background and cards no longer paint an opaque background, so it stays dim behind the tiles.
- M3: the pinned move is seed 42 + Up; frame 14 carries the ring and sparks for the captures.
- M4: `Shell` queues at most one `warning: <store error>` line; `main` prints it after `run_live` returns, so the guard has restored the terminal (N3).
- M4: the A3 child re-executes the test binary via `current_exe()` and resolves the real default path with `XDG_DATA_HOME` set only in `Command::env`; the parent never mutates its own environment.
- M4: `make e2e` writes the three captures under `docs/evidence/M4/` and `diff -u`s each against its committed fixture, so drift fails the target.

## Known issues
- The agent shell is headless (no tty); live truecolour and the plasma shimmer are still the human's visual check (Q1/N5).
- The pty reports terminal sizes that are not the target 100x36; the TestBackend snapshots carry the visible-layout proof.
- The live measured-`dt` loop compiles and is wired but was not exercised under a real tty here; the deterministic frame tests carry the animation proof.
- Human checks remaining before final acceptance: truecolour shimmer and effect frame 14, live cursor/screen restore, and BEST surviving a real restart via `make run`.
