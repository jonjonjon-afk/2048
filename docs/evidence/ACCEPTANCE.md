# ACCEPTANCE — current proof for A1-A11

> Written by the coder for M12. Each block is REAL output appended by
> `scripts/ev.sh M12 "<label>" -- <command>` (raw log: `docs/evidence/M12.md`).
> Output is filtered to the lines that prove the row (test-result, pty or
> clean-clone summary lines), not a full suite dump. Date is UTC.

### A1 — Layout at 100x36 and 60x30

**Tests:** `render_100x36_matches_the_snapshot`, `render_60x30_matches_the_snapshot`, `card_geometry_is_9x5_with_one_cell_gaps`, `too_small_terminal_shows_a_notice`

**Expected:** Both snapshots render exactly; cards are 9x5 with one-cell gaps; a terminal below 60x30 shows the notice.

**Proof label:** `ACC A1 layout and snapshots` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'for t in render_100x36_matches_the_snapshot render_60x30_matches_the_snapshot card_geometry_is_9x5_with_one_cell_gaps too_small_terminal_shows_a_notice; do cargo test --offline --quiet --lib "render::tests::$t" -- --exact 2>&1 | grep "^test result"; done'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```
date: 2026-11-13T16:12:32Z

### A2 — Every key and the blocking rules

**Tests:** `input::tests` (10), `update::tests` (12)

**Expected:** Every §5.5 key maps, release events are ignored, and moves are blocked by the win overlay, game over, the table overlay and name entry.

**Proof label:** `ACC A2 keys and blocking` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib input::tests 2>&1 | grep "^test result"; cargo test --offline --quiet --lib update::tests 2>&1 | grep "^test result"'
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 89 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s

[exit code: 0]
```
date: 2026-11-13T16:12:35Z

### A3 — High score across a process restart; no-persist isolation

**Tests:** `named_entry_survives_a_process_restart`, `no_persist_never_writes_the_real_path`

**Expected:** Two child processes read and write `$XDG_DATA_HOME/2048/highscores`; no-persist leaves neither file under a temp XDG.

**Proof label:** `ACC A3 restart and no-persist` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test restart named_entry_survives_a_process_restart -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli no_persist_never_writes_the_real_path -- --exact 2>&1 | grep "^test result"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

[exit code: 0]
```
date: 2026-11-13T16:12:35Z

### A4 — Merge effects visible and deterministic

**Tests:** `ripple_*` (8), `effect_snapshots_match_frames_4_14_24`; purity tests in the same suite

**Expected:** The ripple, sparks, flash, pop and pulse follow their fixed-`dt` lifetimes; frames 4/14/24 match the fixtures; effects never change the game or RNG.

**Proof label:** `ACC A4 deterministic effects` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test effects ripple_ 2>&1 | grep -E "^test ripple_|^test result"; cargo test --offline --quiet --test cli effect_snapshots_match_frames_4_14_24 -- --exact 2>&1 | grep "^test result"'
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.01s

[exit code: 0]
```
date: 2026-11-13T16:12:37Z

### A5 — Terminal restore and pty self-test

**Tests:** `terminal::tests` (3), `--self-test-tui --fps 60` under `script -qec`

**Expected:** Restore runs exactly once after a normal drop and after a simulated panic; the pty self-test exits 0.

**Proof label:** `ACC A5 restore and pty self-test` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib terminal::tests 2>&1 | grep "^test result"; script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null | tr -d "\r" | grep -o "self-test-tui:.*"'
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out; finished in 0.00s
self-test-tui: 120 frames in 2.00s (60 fps)

[exit code: 0]
```
date: 2026-11-13T16:12:39Z

### A6 — Clean-clone `make check` + `make e2e`

**PENDING** — block `ACC A6 clean-clone gate` not captured yet.

### A7 — Card geometry and label row

**Tests:** `card_geometry_is_9x5_with_one_cell_gaps`, `tile_label_is_centred_on_the_middle_row`, both snapshots

**Expected:** The play area is 41x25, cards are 9x5 with one-cell gaps, and each tile value sits on the card middle row, centred within one cell.

**Proof label:** `ACC A7 card geometry and label row` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'for t in card_geometry_is_9x5_with_one_cell_gaps tile_label_is_centred_on_the_middle_row render_100x36_matches_the_snapshot render_60x30_matches_the_snapshot; do cargo test --offline --quiet --lib "render::tests::$t" -- --exact 2>&1 | grep "^test result"; done'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```
date: 2026-11-13T16:12:42Z

### A8 — 60 FPS deadline pacing

**Tests:** `frame_pacing_skips_missed_frames`, `--self-test-tui --fps 60` under `script -qec`

**Expected:** The deadline helper skips missed frames; the pty self-test prints the measured 60 fps line for 120 frames.

**Proof label:** `ACC A8 60 FPS pacing` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib shell::tests::frame_pacing_skips_missed_frames -- --exact 2>&1 | grep "^test result"; script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null | tr -d "\r" | grep -o "self-test-tui:.*"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
self-test-tui: 120 frames in 2.00s (60 fps)

[exit code: 0]
```
date: 2026-11-13T16:12:43Z

### A9 — Score count-up is display-only

**Tests:** `score::tests` (2), `tests/score.rs` (7), `dump_prints_the_real_score`

**Expected:** The display follows G14/G15, retargets from the shown value, resets on restart, leaves BEST immediate, and `--dump` prints the real score.

**Proof label:** `ACC A9 score count-up` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --lib score::tests 2>&1 | grep "^test result"; cargo test --offline --quiet --test score 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli dump_prints_the_real_score -- --exact 2>&1 | grep "^test result"'
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.01s

[exit code: 0]
```
date: 2026-11-13T16:12:46Z

### A10 — Round, half-block ripple

**Tests:** `ripple_*` (8), `effect_snapshots_match_frames_4_14_24`, frame-14 `▀`/`▄` counts

**Expected:** The front is aspect-corrected Euclidean, sampled at `y ± 0.25`, drawn as `▀`/`▄` with fg/bg, reflects off a wall and fades before 900 ms; frame 14 has 8 `▀` lines and 4 `▄` lines.

**Proof label:** `ACC A10 round half-block ripple` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test effects ripple_ 2>&1 | grep "^test result"; cargo test --offline --quiet --test cli effect_snapshots_match_frames_4_14_24 -- --exact 2>&1 | grep "^test result"; cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -c "▀"; cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -c "▄"'
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.01s
8
4

[exit code: 0]
```
date: 2026-11-13T16:12:46Z

### A11 — A named score is stored and shown after restart

**Tests:** `named_entry_survives_a_process_restart`, `entry_enter_saves_the_real_score_and_emits_one_save`, `entry_overlay_shows_the_counted_score_and_the_typed_name`, `high_score_panel_is_20x12_at_100x36`

**Expected:** Enter saves the real game score, the row reloads in a second process, and the panel/entry overlay render the stored name.

**Proof label:** `ACC A11 named score persists` (in `docs/evidence/M12.md`)

```text
$ bash -o pipefail -c 'cargo test --offline --quiet --test restart named_entry_survives_a_process_restart -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib update::tests::entry_enter_saves_the_real_score_and_emits_one_save -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib render::tests::entry_overlay_shows_the_counted_score_and_the_typed_name -- --exact 2>&1 | grep "^test result"; cargo test --offline --quiet --lib render::tests::high_score_panel_is_20x12_at_100x36 -- --exact 2>&1 | grep "^test result"'
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 0.00s

[exit code: 0]
```
date: 2026-11-13T16:12:48Z
