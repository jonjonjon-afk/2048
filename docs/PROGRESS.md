# PROGRESS

> Owned by the coder. Compact state after M14 round 1; the entry point is
> `docs/HANDOVER.md`.

## M14 plan (round 1)
1. Commit the reviewer's SPEC/M14 docs (and the dirty M13 approval) as-is.
2. R1: `anim::jiggle_offset(age, direction)` and the G22 per-direction tests.
3. R2: `Jiggle`/`SlideState` carry the move `Direction`; `App::jiggle_offset`.
4. R3: draw the card and label with `(dx, dy)` on both axes, clamped.
5. R4: regenerate the three wobble fixtures; fix the ripple test comment/rect.
6. R5: HANDOVER, A12 and PROGRESS record the merge axis and the real gate.
7. R6: `ev.sh` labels R1-R6 with the fixture diffs and the exact gate.

## M13 final state (round 3 — F6 only)
1. `demo_slide_timeline_is_pinned` now pins `last_spawn == Some(Spawn { cell: 0, value: 2 })`, hand-derived from SPEC 5.2 with seed 42.
2. The evidence map's GATE and F5 rows now say what happened: round 2 had no gate block (F5 deviated); round 3 adds it.
3. The test + docs were committed at `20b922b`, so the gate ran on a clean tree.
4. The exact M13 gate from `docs/reviews/M13.md` line 13 ran through `scripts/ev.sh` label "M13 gate".
5. Result: **exit 0**, **181 tests, 0 failed**; the block is committed.

## Current state
- M0-M13 are approved; M14 (jiggle along the merge axis) is at its round-1 gate.
- `make check`: **185 tests, 0 failed** (121 lib + 15 CLI + 41 effects + 1
  restart + 7 score). `make e2e` is clean against the four slide and six older
  fixtures.
- M14 gate: **exit 0** at commit `0bd8448` (clean source tree): `make check`,
  the seed-42 dump, the frame 3/9/12 slide diffs and `make e2e`. Proof:
  `docs/evidence/M14.md` "M14 gate".
- M13 gate: **exit 0** at commit `20b922b` (clean tree): `make check`, the
  seed-42 dump, the frame 3/9/12 slide diffs and `make e2e`. Proof:
  `docs/evidence/M13.md` "M13 gate".
- M12 gate: **exit 0** at commit `bdd89a2` round 2 (147/0). Proof:
  `docs/evidence/M12.md` "M12 round 2 gate".
- No next action: stop at the M14 gate and await the reviewer's verdict.

## Milestones
| ID | Title | Verdict | Final commit | History |
|---|---|---|---|---|
| M0 | Recon and plan | approved with follow-ups | `74d7b1a` | docs/HISTORY.md |
| M1 | Headless core and storage | approved | `bb7f5a9` | docs/HISTORY.md |
| M2 | TUI shell and controls | approved | `0a00859` | docs/HISTORY.md |
| M3 | Shader-style effects | approved with follow-ups | `f9bd22d` | docs/HISTORY.md |
| M4 | Polish and handover | approved with follow-ups | `641a5b1` | docs/HISTORY.md |
| M5 | Prettier board and real 60 FPS | approved | `a4264fa` | docs/HISTORY.md |
| M6 | Score count-up | approved with follow-ups | `cb1fd7e` | docs/HISTORY.md |
| M7 | Merge ripple | approved (round 2) | `e1a86c1` | docs/HISTORY.md |
| M8 | Arcade high-score table | approved (round 2) | `ca53b8c` | docs/HISTORY.md |
| M9 | Handover refresh | approved (round 2) | `9e22a51` | docs/HISTORY.md |
| M10 | Round merge ripple | approved (round 2) | `6adddbe` | docs/HISTORY.md |
| M11 | Half-block ripple | approved (round 2) | `9f2f25e` | docs/HISTORY.md |
| M13 | Tile slide and fusion jiggle | approved (round 3) | `b063ce3` | docs/evidence/M13.md |
| M14 | Jiggle along the merge axis | awaiting round-1 review | `0bd8448` | docs/evidence/M14.md |

## M12 checklist
| ID | State |
|---|---|
| R1 | docs consolidated, old logs deleted, `/docs/evidence/e2e/` ignored |
| R2 | `docs/HANDOVER.md` is the entry point (125 lines) |
| R3 | `docs/HISTORY.md` has 12 milestone rows M0-M11 |
| R4 | `docs/evidence/ACCEPTANCE.md` has A1-A11 with fresh ev.sh blocks |
| R5 | standing docs trimmed; README links HANDOVER/HISTORY/EVIDENCE/api-notes/OPEN_QUESTIONS |
| R6 | docs-only diff; fixtures, code, Makefile and Cargo untouched; 147 tests |
| R7 | before/after doc size recorded in `docs/evidence/M12.md` |
| R8 | workflow files, templates, tests and src untouched; SPEC diff is the M12 line |
| Gate | exact SPEC §8 command exit 0 at `bdd89a2` round 2 (proof: `docs/evidence/M12.md` "M12 round 2 gate") |
