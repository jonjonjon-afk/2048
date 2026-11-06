# EVIDENCE — requirement / acceptance ID -> proof

> Owned by the coder. Update at the end of every milestone. One row per requirement ID (R1.. from docs/reviews/M<n>.md) and per acceptance ID (A1.. from docs/SPEC.md section 9).

| ID | Test name(s) | Command | Result | Date (UTC) | Proof (file + log section label) |
|---|---|---|---|---|---|
| R1 | api-notes data-row/status audit (docchecks.sh) | `bash scripts/ev.sh M0 "intake doc checks" -- bash -c '... docchecks.sh ...'` | 21 data rows, 18 VERIFIED, 3 UNVERIFIED, 0 empty Source, 0 invalid Status; ratatui/crossterm/directories all present | 2026-11-06 | docs/evidence/M0.md "intake doc checks" |
| R2 | PROGRESS milestone table and plan count | same docchecks.sh run | 5 M0-M4 rows, 7 plan bullet lines, current state names M0 | 2026-11-06 | docs/evidence/M0.md "intake doc checks" |
| R3 | OPEN_QUESTIONS default/Blocks audit | same docchecks.sh run | 6 data rows, 0 empty Proposed default, 0 empty Blocks work? | 2026-11-06 | docs/evidence/M0.md "intake doc checks" |
| R4 | local crate source inspection + compiled probe | `cargo tree --offline --depth 1`; `crate versions`; `api source checks`; `cargo run --offline` in /tmp/opencode/m0-probe | ratatui 0.30.2, crossterm 0.29.0, directories 6.0.0, ratatui-core 0.1.2; all named APIs read in source; probe printed TestBackend 60x30, KeyEvent press, data dir | 2026-11-06 | docs/evidence/M0.md "cargo tree", "crate versions", "api source checks", "project dirs probe" |
| R5 | environment capture | `bash scripts/ev.sh M0 "environment" -- bash -c '...'` | rustc 1.99.1, cargo 1.99.1, GNU Make 4.4.1, script 2.42.4, TERM=xterm-256color, COLORTERM=truecolor, tput colors 256, tput 80x24, tty "not a tty", pty 0 0 | 2026-11-06 | docs/evidence/M0.md "environment" |
| R6 | initial commit, no app code | `git log -1`, `git ls-files`, `git status --porcelain` | commit message starts M0:; no tracked Cargo.toml or src/; status empty at capture time; branch main | 2026-11-06 | docs/evidence/M0.md "commit state" |
| R7 | protected file hashes before/after commit | `sha256sum` of the six files | all six equal docs/reviews/M0.md values (9cbb0cc7..., 8ee70083..., 25c15d9b..., 1854ea1b..., 619c9165..., 3d0e854d...) | 2026-11-06 | docs/evidence/M0.md "protected hashes (pre-commit)", "protected hashes (post-commit)" |
| R8 | final report + label presence | this report read against docs/evidence/M0.md | one line per R1-R8; every cited label exists | 2026-11-06 | docs/evidence/M0.md (all labelled blocks) |
| A1 | snapshots at 100x36 and 60x30 | - | not started (M2) | - | - |
| A2 | key mapping and win/game-over input tests | - | not started (M2) | - | - |
| A3 | high-score restart e2e with isolated XDG_DATA_HOME | - | not started (M1-M2) | - | - |
| A4 | deterministic merge effect frames 4/14/24 | - | not started (M3) | - | - |
| A5 | terminal restore on quit/panic, --self-test-tui | - | not started (M2) | - | - |
| A6 | make check and make e2e green | - | not started (M4) | - | - |
