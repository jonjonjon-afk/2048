# HISTORY — one line per milestone

> Started in M12 and kept current through M15. The per-milestone reviews and raw
> logs were removed from the working tree to keep the handover lean; every one of
> them is still in git history at the commits below
> (`git show <commit>:docs/reviews/M<n>.md`).

| ID | Title | Verdict | Final commit | Gate |
|---|---|---|---|---|
| M0 | Recon and plan | approved with follow-ups | `74d7b1a` | exit 0 |
| M1 | Headless core and storage | approved | `bb7f5a9` | exit 0 |
| M2 | TUI shell and controls | approved | `0a00859` | exit 0 |
| M3 | Shader-style effects | approved with follow-ups | `f9bd22d` | exit 0 |
| M4 | Polish and handover | approved with follow-ups | `641a5b1` | exit 0 |
| M5 | Prettier board and real 60 FPS | approved | `a4264fa` | exit 0 |
| M6 | Score count-up | approved with follow-ups | `cb1fd7e` | exit 0 |
| M7 | Merge ripple | approved (round 2) | `e1a86c1` | exit 0 |
| M8 | Arcade high-score table | approved (round 2) | `ca53b8c` | exit 0 |
| M9 | Handover refresh | approved (round 2) | `9e22a51` | exit 0 |
| M10 | Round merge ripple (post-handover) | approved (round 2) | `6adddbe` | exit 0 |
| M11 | Half-block ripple (post-handover) | approved (round 2) | `9f2f25e` (approval audit `687804b`) | exit 0 |
| M13 | Tile slide and fusion jiggle | approved (round 3) | `b063ce3` | exit 0 |
| M14 | Jiggle along the merge axis | approved | `da033c1` | exit 0 |

## Notes
- The two later ripple changes are M10 (Chebyshev square -> aspect-corrected
  Euclidean circle) and M11 (full-cell glyphs -> `▀`/`▄` half-block subcells);
  both kept the radius, 900 ms lifetime, ±0.75 window, nine wall images, 0.55
  reflection factor and per-channel colour fade.
- Live acceptances by the user: M6 count-up ("score count, looks good"), M7
  linear fade (chosen live from the review options), M8 named table ("go,
  everything looks great."), M10 round crest ("that is looking clean! very
  good."), M11 half blocks ("it looks good!"), M13 slide and jiggle
  ("gameplay feels good now, jiggle and slide is working properly") and M14
  merge axis ("everything is working fine").
- M12 is documentation/repo cleanup only; it changes no application code, test,
  fixture, Makefile or Cargo file, so it stays out of the milestone table.
  M13 (tile slide, key lockout, effects at the landing, merge jiggle) and M14
  (jiggle along the merge axis) are the two motion milestones that follow it.
- After M14: the 140 ms slide and the 240 ms jiggle read from
  `src/anim.rs` (pure geometry over injected ages) and are pinned by the four
  slide fixtures; M15 adds `--size <WxH>` so the 60x30 capture has a documented
  path.
- Commit hashes are the last commit of each acknowledged milestone; the full
  chain is in `git log`.
