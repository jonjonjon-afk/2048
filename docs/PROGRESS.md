# PROGRESS

> Owned by the coder. Compact final state after M12; the entry point is
> `docs/HANDOVER.md`.

## M13 plan (round 3 — fix F6 only)
1. Add the hand-derived `last_spawn == Some(Spawn { cell: 0, value: 2 })` assert to `demo_slide_timeline_is_pinned`.
2. Correct the evidence map's GATE and F5 rows to what is actually true.
3. Commit the test + docs so the gate runs on a clean tree.
4. Run the exact M13 gate from `docs/reviews/M13.md` line 13 through `scripts/ev.sh` label "M13 gate".
5. Record the real test count and gate exit code in PROGRESS and the evidence rows; commit the appended block.

## Current state
- All milestones M0-M11 are approved; the game is feature-complete for the SPEC.
- M12 (lean handover) is documentation/repo cleanup only. The per-milestone
  reviews and raw logs were deleted; `docs/HISTORY.md`, `docs/HANDOVER.md` and
  `docs/evidence/ACCEPTANCE.md` replace them. Git history keeps every old file.
- `make check`: **147 tests, 0 failed** (99 lib + 12 CLI + 28 effects + 1
  restart + 7 score). `make e2e` is clean against the fixtures.
- M12 gate: **exit 0** at commit `bdd89a2` round 2 (local and clean clone 147/0,
  clone `make e2e` green, clone status empty). Proof: `docs/evidence/M12.md`
  "M12 round 2 gate"; pointer check "F3 pointer check" is 11/11, 0 dangling.
- No next action: stop at the M12 gate and await the reviewer's verdict.

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
