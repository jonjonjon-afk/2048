# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M11 (half-block ripple) — gate passed in round 2 (F1/F2 corrections), 147 tests, 0 failed
- Last gate result: M11 gate exit 0 (docs/evidence/M11.md "M11 gate round 2"); 99 lib + 12 CLI + 28 effects + 1 restart + 7 score = 147 tests, 0 failed; two `make e2e` runs exit 0 with `docs/evidence/e2e/` clean
- Q15/Q16 are answered; `tests/cli.rs` changed exactly the two frame-14 ripple assertions (2/2); protected set (without `tests/cli.rs`) 0-line diff
- Next action: none; stop at the M11 gate and await the reviewer's round-2 verdict

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
| M7 | Merge ripple | gate passed | approved (round 2, `e1a86c1`) | docs/evidence/M7.md |
| M8 | Arcade high-score table | gate passed | approved (round 2, `ca53b8c`) | docs/evidence/M8.md |
| M9 | Handover refresh | gate passed | awaiting the reviewer's verdict (round 1) | docs/evidence/M9.md |
| M10 | Round merge ripple | gate passed (round 2) | approved (round 2, `6adddbe`) | docs/evidence/M10.md |
| M11 | Half-block ripple | gate passed (round 2) | awaiting the reviewer's verdict (round 2) | docs/evidence/M11.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- Commit the reviewer-owned `docs/SPEC.md` M12 line and `docs/reviews/M12.md` unchanged first.
- Record the before size of the docs (R7) and create `docs/HISTORY.md` (12 milestone rows) and `docs/HANDOVER.md`.
- Delete `docs/evidence/M0..M11*`, `docs/reviews/M0..M11.md`, `docs/evidence/M3/`, `M4/`, `e2e/`; ignore `/docs/evidence/e2e/`.
- Build `docs/evidence/ACCEPTANCE.md`: A1-A11 with concise ev.sh blocks (test-result and pty lines).
- Trim `EVIDENCE.md` (A rows only), `PROGRESS.md`, `OPEN_QUESTIONS.md` (<=13 rows) and `api-notes.md`.
- Update README's detail links to HANDOVER/HISTORY/EVIDENCE/api-notes/OPEN_QUESTIONS.
- Audit R6/R8: docs-only diff, protected files, SPEC one-line diff; record after size (R7).
- Commit everything, run the exact M12 gate through ev.sh, fill evidence maps, report.

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
- M7 round 2: `Ripple::front_at` returns the glyph *and* the amplitude that selected it, so `draw_ripple` cannot dim by a different value than the one that chose the glyph.
- M7 round 2: the brightness mix lives in the renderer (`dimmed_colour`), because the background is renderer-owned; the effect layer stays colour-free.
- M7 round 2: `f64::round` is used per channel (round half away from zero), matching the four literals `Rgb(1,152,197)`, `Rgb(4,87,117)`, `Rgb(3,98,130)`, `Rgb(5,51,72)` in the review.
- M7 round 2: the text fixtures and the e2e capture needed no regeneration (colour is not in the plain-text frame); they have a 0-line diff from `d6909ef`.
- M8 round 2 (F1): only `ErrorKind::NotFound` may fall through to the legacy migration; any other table read error returns an empty table, so a directory at the table path does not promote a stale legacy score.
- M10: `RIPPLE_ROW_ASPECT = 2.0` is a named `f64` constant and `amplitude_at` computes `sqrt(dx*dx + (aspect*dy)^2)`, because SPEC §5.4 replaces the Chebyshev square with the aspect-corrected circle.
- M10 round 2 (F1): the rendered reflected crest is pinned at `(2,5)` (`▒`, `Rgb(4,87,117)`) while the effect layer keeps `front_at(3,2) = ('▒', 0.413111)`; `(3,2)` is the value-4 label row and a spark cell, so only the reviewer's pin changed, not the code.

## Known issues
- The agent shell is headless (no tty); live truecolour and the plasma shimmer are still the human's visual check (Q1/N5).
- The pty reports terminal sizes that are not the target 100x36; the TestBackend snapshots carry the visible-layout proof.
- The live measured-`dt` loop compiles and the pty self-test exercises the same deadline pacing, but live keyboard input under a real tty was not exercised here; the deterministic frame tests carry the animation proof.
- M5 round 2 changed documentation only; the geometry stays flush 41x25 with a 60x30 minimum (F5 resolved by the human's accepted build).
- M6: the count-up *feel* was watched live by the human ("score count, looks good") and accepted; the fixed-`dt` tests and G15 frames remain the proof here.
- M7: the ripple *feel* was not watched live by the coder; the pinned cells, the hand table and frame 14 are the proof, and the human's live wave/return/fade check is open.
- M7 round 2: the linear fade is proven per-channel in the buffer, but how it feels over the 900 ms at 60 FPS is the human's live check; the text fixtures cannot show colour.
- M7: the label layer paints its whole 7-cell row, so a ripple glyph on an occupied card's label row takes that tile's foreground colour; digits still win in their own cell and no pin sits on a label row.
- M10: `(3,2)` is both the value-4 label row and a spark cell at 224 ms, so the rendered reflected crest is pinned at `(2,5)` (Q14); the effect layer keeps the exact `front_at(3,2)` value.
- Human checks remaining before final acceptance: truecolour shimmer and effect frame 14, the M7 ripple reading as a returning wave, live cursor/screen restore, and BEST surviving a real restart via `make run`.
