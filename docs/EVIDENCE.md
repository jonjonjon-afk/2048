# EVIDENCE — acceptance ID -> proof

> Owned by the coder. Current proof lives in `docs/evidence/ACCEPTANCE.md`
> (fresh A1-A11 runs, 2026-11-13). Historical per-milestone logs were removed in
> M12 but remain in git history (`docs/HISTORY.md` lists the commits).

| ID | What it proves | Current tests | Proof (ACCEPTANCE.md label) |
|---|---|---|---|
| A1 | 100x36 and 60x30 render without panic, overlap or truncation; too-small notice | `render_100x36_matches_the_snapshot`, `render_60x30_matches_the_snapshot`, `card_geometry_is_9x5_with_one_cell_gaps`, `too_small_terminal_shows_a_notice` | "ACC A1 layout and snapshots" |
| A2 | every §5.5 key, including win/game-over/table/name-entry blocking | `input::tests` (10), `update::tests` (12) | "ACC A2 keys and blocking" |
| A3 | named score survives a process restart through `$XDG_DATA_HOME/2048/highscores`; no-persist writes nothing | `named_entry_survives_a_process_restart`, `no_persist_never_writes_the_real_path` | "ACC A3 restart and no-persist" |
| A4 | merge effects visible and deterministic at pinned frames; effects never change rules or RNG | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24` | "ACC A4 deterministic effects" |
| A5 | terminal restored after normal quit and simulated panic; self-test exits 0 | `terminal::tests` (3), `--self-test-tui` pty line | "ACC A5 restore and pty self-test" |
| A6 | `make check` + `make e2e` green, clean clone committed and clean | clean-clone summary (147 tests, 0 failed; e2e exit 0; status empty) | "ACC A6 clean-clone gate" |
| A7 | play-area 41x25, cards 9x5 with one-cell gaps, values on the middle row, centred | `card_geometry_is_9x5_with_one_cell_gaps`, `tile_label_is_centred_on_the_middle_row`, both snapshots | "ACC A7 card geometry and label row" |
| A8 | live loop and self-test pace to the 60 FPS deadline | `frame_pacing_skips_missed_frames`, pty measured `60 fps` line | "ACC A8 60 FPS pacing" |
| A9 | count-up follows G14/G15, retargets, resets on restart, BEST immediate, dump real | `score::tests` (2), `tests/score.rs` (7), `dump_prints_the_real_score` | "ACC A9 score count-up" |
| A10 | ripple round (aspect-corrected), half-block subcells, wall reflection, fade, deterministic frames | `ripple_*` (8), `effect_snapshots_match_frames_4_14_24`, frame 14 `▀`/`▄` counts | "ACC A10 round half-block ripple" |
| A11 | a named score entered at game over is saved, reloaded and shown after restart | `named_entry_survives_a_process_restart`, `entry_enter_saves_the_real_score_and_emits_one_save`, `entry_overlay_shows_the_counted_score_and_the_typed_name`, `high_score_panel_is_20x12_at_100x36` | "ACC A11 named score persists" |
