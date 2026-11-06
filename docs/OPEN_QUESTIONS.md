# OPEN QUESTIONS

> Owned by the coder. Every question gets a proposed default so work can continue. The reviewer relays open questions to the human and records the answers in docs/reviews/M<n>.md under "Human decisions".

| # | Question | Proposed default | Raised in | Blocks work? | Answer |
|---|---|---|---|---|---|
| Q1 | The live 24-bit neon look cannot be seen from the headless agent shell (no tty, pty 0x0), so visual polish is unverified here. | Verify layout and colours through TestBackend snapshots at 100x36 and 60x30; the human does one live visual check at M4. | M0 | No | - |
| Q2 | `XDG_DATA_HOME` may be unset or relative on the user's machine; directories 6 silently falls back to `~/.local/share` when it is not absolute. | Live play uses `ProjectDirs::from("", "", "2048")`; every test and e2e run sets an absolute temporary `XDG_DATA_HOME`. | M0 | No | - |
| Q3 | The crates.io API returned HTTP 403 from this shell; only the already-cached crates are guaranteed to resolve. | Build from the local registry cache; use `--offline` only if the index refresh fails; ask the reviewer before adding any dependency. | M0 | No for M1-M3; yes if a new dependency is needed | - |
| Q4 | SPEC §5.6 says `--demo-merge` shows "demo board plus an immediate merge effect" but does not pin the board, the merge cell or the effect start frame. | Use the seed-42 demo board, perform one fixed documented merge, and make the effect visible at frames 4/14/24 as required by the M3 gate. | M0 | No | - |
| Q5 | SPEC §5.3 does not say exactly when the best score is written (every new best, on quit, or both). | Save atomically on every new best and on clean quit; a failed save keeps the game running and shows the footer warning. | M0 | No | - |
| Q6 | The win overlay blocks move keys (SPEC §5.5) but the behaviour of R and Q while it is visible is unstated. | Q quits and R restarts from the win overlay; only the four move keys are ignored until Space/Enter dismisses it. | M0 | No | - |
