# ACCEPTANCE — raw A1-A11 acceptance log

> Written by the coder with `scripts/ev.sh ACCEPTANCE "<label>" -- <command>`.
> Every block below the map is real, unfiltered-at-write-time ev.sh output:
> `### <label>`, `date:`, a fenced `$ ...` block and `[exit code: N]`. Nothing
> is hand-pasted. The commands filter to the proving lines (test-result, pty or
> clean-clone summary) so this stays short; the cleanup/gate log is
> `docs/evidence/M12.md`. `docs/EVIDENCE.md` points at the labels here.

## Map
| ID | Tests / check | Expected | Raw-log label |
|---|---|---|---|
| A1 | `render_100x36_matches_the_snapshot`, `render_60x30_matches_the_snapshot`, `card_geometry_is_9x5_with_one_cell_gaps`, `too_small_terminal_shows_a_notice` | both snapshots exact; 9x5 cards with one-cell gaps; below 60x30 the notice shows | `ACC A1 layout and snapshots` |
| A2 | `input::tests` (10), `update::tests` (12) | every §5.5 key maps; win/game-over/table/name-entry blocking holds | `ACC A2 keys and blocking` |
| A3 | `named_entry_survives_a_process_restart`, `no_persist_never_writes_the_real_path` | two processes use `$XDG_DATA_HOME/2048/highscores`; no-persist writes nothing | `ACC A3 restart and no-persist` |
| A4 | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24` | effects deterministic at pinned frames; rules and RNG unchanged | `ACC A4 deterministic effects` |
| A5 | `terminal::tests` (3), `--self-test-tui --fps 60` pty line | restore exactly once after drop and panic; self-test exits 0 | `ACC A5 restore and pty self-test` |
| A6 | clean clone: `make check`, `make e2e`, `git status --porcelain` | 147 tests, 0 failed; e2e exit 0; clone status empty | `ACC A6 clean-clone gate` |
| A7 | `card_geometry_is_9x5_with_one_cell_gaps`, `tile_label_is_centred_on_the_middle_row`, both snapshots | play area 41x25; cards 9x5; values on the middle row, centred | `ACC A7 card geometry and label row` |
| A8 | `frame_pacing_skips_missed_frames`, `--self-test-tui --fps 60` pty line | deadline pacing; measured 60 fps for 120 frames | `ACC A8 60 FPS pacing` |
| A9 | `score::tests` (2), `tests/score.rs` (7), `dump_prints_the_real_score` | G14/G15 count-up, retarget, restart reset, BEST immediate, dump real | `ACC A9 score count-up` |
| A10 | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24`, frame-14 `▀`/`▄` counts | round aspect-corrected half-block front, wall reflection, fade before 900 ms | `ACC A10 round half-block ripple` |
| A11 | `named_entry_survives_a_process_restart`, `entry_enter_saves_the_real_score_and_emits_one_save`, `entry_overlay_shows_the_counted_score_and_the_typed_name`, `high_score_panel_is_20x12_at_100x36` | entered name saved, reloaded and shown after restart | `ACC A11 named score persists` |

## Command log

### ACC A1 layout and snapshots
date: 2026-11-13T16:17:22Z
```text
$ bash -o pipefail -c 'for t in render_100x36_matches_the_snapshot render_60x30_matches_the_snapshot card_geometry_is_9x5_with_one_cell_gaps too_small_terminal_shows_a_notice; do cargo test --offline --quiet --lib "render::tests::$t" -- --exact 2>&1 | grep "^test result"; done'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A2 keys and blocking
date: 2026-11-13T16:17:22Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib input::tests 2>&1 | grep "^test result"; cargo test --offline --quiet --lib update::tests 2>&1 | grep "^test result"'
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A3 restart and no-persist
date: 2026-11-13T16:17:22Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test restart named_entry_survives_a_process_restart -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli no_persist_never_writes_the_real_path -- --exact 2>&1 | grep "^test result"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A4 deterministic effects
date: 2026-11-13T16:17:24Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test effects ripple_ 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli effect_snapshots_match_frames_4_14_24 -- --exact 2>&1 | grep "^test result"'
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.01s

[exit code: 0]
```

### ACC A5 restore and pty self-test
date: 2026-11-13T16:17:27Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib terminal::tests 2>&1 | grep "^test result"; script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null | tr -d "\r" | grep -o "self-test-tui:.*"'
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 0.00s
self-test-tui: 120 frames in 2.00s (60 fps)

[exit code: 0]
```

### ACC A7 card geometry and label row
date: 2026-11-13T16:17:27Z
```text
$ bash -o pipefail -c 'for t in card_geometry_is_9x5_with_one_cell_gaps tile_label_is_centred_on_the_middle_row render_100x36_matches_the_snapshot render_60x30_matches_the_snapshot; do cargo test --offline --quiet --lib "render::tests::$t" -- --exact 2>&1 | grep "^test result"; done'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A8 60 FPS pacing
date: 2026-11-13T16:17:33Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib shell::tests::frame_pacing_skips_missed_frames -- --exact 2>&1 | grep "^test result"; script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null | tr -d "\r" | grep -o "self-test-tui:.*"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
self-test-tui: 120 frames in 2.00s (60 fps)

[exit code: 0]
```

### ACC A9 score count-up
date: 2026-11-13T16:17:33Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib score::tests 2>&1 | grep "^test result"; cargo test --offline --quiet --test score 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli dump_prints_the_real_score -- --exact 2>&1 | grep "^test result"'
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A10 round half-block ripple
date: 2026-11-13T16:17:33Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test effects ripple_ 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli effect_snapshots_match_frames_4_14_24 -- --exact 2>&1 | grep "^test result"; cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -c "▀"; cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -c "▄"'
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.01s
8
4

[exit code: 0]
```

### ACC A11 named score persists
date: 2026-11-13T16:17:33Z
```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test restart named_entry_survives_a_process_restart -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib update::tests::entry_enter_saves_the_real_score_and_emits_one_save -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib render::tests::entry_overlay_shows_the_counted_score_and_the_typed_name -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib render::tests::high_score_panel_is_20x12_at_100x36 -- --exact 2>&1 | grep "^test result"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```

### ACC A6 clean-clone gate
date: 2026-11-13T16:17:44Z
```text
$ bash -o pipefail -c 'rm -rf /tmp/2048-clean && git clone -q . /tmp/2048-clean && cd /tmp/2048-clean && make check 2>&1 | grep -E "^(test result|error)" && make e2e >/dev/null 2>&1 && echo "make e2e: exit 0" && test -z "$(git status --porcelain)" && echo "clone git status: clean"'
test result: ok. 99 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
make e2e: exit 0
clone git status: clean

[exit code: 0]
```
