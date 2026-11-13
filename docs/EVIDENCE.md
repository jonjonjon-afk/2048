# EVIDENCE — acceptance ID -> proof

> Owned by the coder. Current proof lives in `docs/evidence/ACCEPTANCE.md`, which
> is the raw `scripts/ev.sh` acceptance log (labels `ACC A1` ... `ACC A11`).
> The per-milestone logs are `docs/evidence/M12.md` (cleanup), `M13.md`
> (slide/jiggle), `M14.md` (merge axis) and `M15.md` (60x30 capture path and
> docs refresh); older ones remain in git history (`docs/HISTORY.md` lists the
> commits). Current suite: 190 tests, 0 failed (124 lib + 17 CLI + 41 effects
> + 1 restart + 7 score).

| ID | What it proves | Current tests | Proof (raw-log label) |
|---|---|---|---|
| A1 | 100x36 and 60x30 render without panic, overlap or truncation; too-small notice | `render_100x36_matches_the_snapshot`, `render_60x30_matches_the_snapshot`, `card_geometry_is_9x5_with_one_cell_gaps`, `too_small_terminal_shows_a_notice` | "ACC A1 layout and snapshots" |
| A2 | every §5.5 key, including win/game-over/table/name-entry blocking | `input::tests` (10), `update::tests` (12) | "ACC A2 keys and blocking" |
| A3 | named score survives a process restart through `$XDG_DATA_HOME/2048/highscores`; no-persist writes nothing | `named_entry_survives_a_process_restart`, `no_persist_never_writes_the_real_path` | "ACC A3 restart and no-persist" |
| A4 | merge effects visible and deterministic at pinned frames; effects never change rules or RNG | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24` | "ACC A4 deterministic effects" |
| A5 | terminal restored after normal quit and simulated panic; self-test exits 0 | `terminal::tests` (3), `--self-test-tui` pty line | "ACC A5 restore and pty self-test" |
| A6 | `make check` + `make e2e` green, clean clone committed and clean; current suite 190 tests (124 lib + 17 CLI + 41 effects + 1 restart + 7 score) | `make check`, `make e2e` | "ACC A6 clean-clone gate"; "M15 gate" in `docs/evidence/M15.md` |
| A7 | play-area 41x25, cards 9x5 with one-cell gaps, values on the middle row, centred | `card_geometry_is_9x5_with_one_cell_gaps`, `tile_label_is_centred_on_the_middle_row`, both snapshots | "ACC A7 card geometry and label row" |
| A8 | live loop and self-test pace to the 60 FPS deadline | `frame_pacing_skips_missed_frames`, pty measured `60 fps` line | "ACC A8 60 FPS pacing" |
| A9 | count-up follows G14/G15, retargets, resets on restart, BEST immediate, dump real | `score::tests` (2), `tests/score.rs` (7), `dump_prints_the_real_score` | "ACC A9 score count-up" |
| A10 | ripple round (aspect-corrected), half-block subcells, wall reflection, fade, deterministic frames | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24`, frame 14 `▀`/`▄` counts | "ACC A10 round half-block ripple" |
| A11 | a named score entered at game over is saved, reloaded and shown after restart | `named_entry_survives_a_process_restart`, `entry_enter_saves_the_real_score_and_emits_one_save`, `entry_overlay_shows_the_counted_score_and_the_typed_name`, `high_score_panel_is_20x12_at_100x36` | "ACC A11 named score persists" |
| A12 | tiles glide on the 140 ms ease-out curve, move keys wait for the slide, merge/spawn effects start at the landing, each merged card wobbles 240 ms along the merge axis (columns for left/right, rows for up/down), and the pinned demo captures stay settled (M13, axis M14) | `anim::tests` (6), `update::tests` (6 slide/jiggle tests), `a_sliding_card_is_drawn_between_its_slots`, `a_merged_card_jiggles_by_the_documented_offsets`, `the_ripple_crosses_a_jiggling_card`, `demo_slide_timeline_is_pinned`, `slide_frames_match_the_fixtures`, `the_live_demo_app_animates_from_frame_zero` | "R1 slide tracks" ... "M13 gate" in `docs/evidence/M13.md`; "R1"-"R6" in `docs/evidence/M14.md`; "R2 60x30 fixture and 100x36 default", "M15 gate" in `docs/evidence/M15.md` |
