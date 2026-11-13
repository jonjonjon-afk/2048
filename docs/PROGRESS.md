# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M7 (merge ripple) — gate passed, awaiting verdict (round 1)
- Last gate result: gate exit 0 (docs/evidence/M7.md "M7 gate final"); 125 tests, 0 failed
- Next action: stop; the human's live check (does the ripple read as a wave that crosses the board, returns from the walls and fades) is still open

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, approved with follow-ups | approved with follow-ups | docs/evidence/M0.md |
| M1 | Headless core and storage | gate passed, approved | approved | docs/evidence/M1.md |
| M2 | TUI shell and controls | gate passed | approved | docs/evidence/M2.md |
| M3 | Shader-style effects | gate passed | approved with follow-ups | docs/evidence/M3.md |
| M4 | Polish and handover | gate passed | approved with follow-ups | docs/evidence/M4.md |
| M5 | Prettier board and real 60 FPS | gate passed | approved | docs/evidence/M5.md |
| M6 | Score count-up | gate passed | approved with follow-ups | docs/evidence/M6.md |
| M7 | Merge ripple | gate passed | awaiting verdict (round 1) | docs/evidence/M7.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- M7 round 1, R1/R7: replace `Shockwave` with `Ripple` in `src/effects.rs` (float radius, 9 images, ±0.75, 0.55^walls) and delete the old ring code.
- M7 R4: one ripple per merge keeps its own cell/value; sparks/flash/pop/pulse lifetimes untouched.
- M7 R2/R3: draw the ripple after cards, before labels; amplitudes via `glyph_at`; expire at 900 ms.
- M7 R1/R2/R3 pins: integration tests read play-area cells at `panel.left+1+x`, `panel.top+1+y`.
- M7 R5: frame 1/2 keep the `4` digit at board cell (4,2); frame 14 shows the ripple at (13,10).
- M7 R6: regenerate the three effect fixtures and the demo-merge capture from the binary; keep frame 14 ▓/▒.
- M7 R7: no `game.score` in render, protected diff empty, no deps, no `Shockwave`/`RING_SYMBOLS` remnants.
- M7 R8: `scripts/ev.sh` evidence, EVIDENCE A10 row, api-notes FU1 rows, PROGRESS without an approval claim.
- Done this round: all eight requirements; gate exit 0 ("M7 gate final"); stop and wait for the audit.

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
- M5: the play-area edge text starts at `panel.left + 2` (a `─` sits at `+1`), because R2 fixes that column and the block corners must stay intact.
- M5: `make e2e` captures live in `docs/evidence/e2e/` from now on; the M4 captures are history and stay byte-unchanged (R6).
- M6: the count is its own module (`src/score.rs`, `ScoreCount`), because "displayed score" is a separate invariant from the rules and the effects; it is pure state advanced only by `dt`.
- M6: `App::new` seeds the count at `game.score`, so a game loaded with a score/best renders its real numbers instead of counting up from 0.
- M6: the count retargets after the action and before `advance(dt)`, so the same `update` both raises the score and advances the count (SPEC §5.7, G15).
- M6: `shown()` snaps when `to <= from`, clamps `elapsed` at 450 ms and caps the counted delta, so no path can display past the real score.
- M6: the three effect fixtures and the demo-merge e2e capture were regenerated by the binary; each diff is exactly one line (the `SCORE` digit).
- M7: `Ripple` lives in `src/effects.rs` with the SPEC's nine images written literally (`-1 - s`, `77 - s`, `-1 - s`, `45 - s`), so the wall geometry is reviewable against §5.4 line by line.
- M7: the radius is `age_ms as f64 * 0.04` with no rounding; `as_secs_f64 * 40` would be equivalent but §5.4 is written in milliseconds (api-notes).
- M7: the wall factor is `0.55` multiplied 0, 1 or 2 times instead of `powi`, because the `powi` docs call its precision unspecified (api-notes).
- M7: glyph choice and colour live behind `Ripple::glyph_at`, so the renderer only writes cells and the pins can be asserted on the effect layer too.
- M7: the ripple is drawn in `render_effects` between `render_board_cards` and `render_board_labels`, which is exactly the §5.4 layering sentence.
- M7: `two_merges_keep_two_waves` triggers the effect layer directly (cells 0 and 5), because one move cannot merge at both cells; a real Left move with cells 0 and 12 covers the update wiring (Q12).

## Known issues
- The agent shell is headless (no tty); live truecolour and the plasma shimmer are still the human's visual check (Q1/N5).
- The pty reports terminal sizes that are not the target 100x36; the TestBackend snapshots carry the visible-layout proof.
- The live measured-`dt` loop compiles and the pty self-test exercises the same deadline pacing, but live keyboard input under a real tty was not exercised here; the deterministic frame tests carry the animation proof.
- M5 round 2 changed documentation only; the geometry stays flush 41x25 with a 60x30 minimum (F5 resolved by the human's accepted build).
- M6: the count-up *feel* was watched live by the human ("score count, looks good") and accepted; the fixed-`dt` tests and G15 frames remain the proof here.
- M7: the ripple *feel* was not watched live by the coder; the pinned cells, the hand table and frame 14 are the proof, and the human's live wave/return/fade check is open.
- M7: the label layer paints its whole 7-cell row, so a ripple glyph on an occupied card's label row takes that tile's foreground colour; digits still win in their own cell and no pin sits on a label row.
- Human checks remaining before final acceptance: truecolour shimmer and effect frame 14, the M7 ripple reading as a returning wave, live cursor/screen restore, and BEST surviving a real restart via `make run`.
