# PROGRESS

> Owned by the coder. This file is the coder's memory between assignments: keep it short and current.

## Current state
- Milestone: M0 (recon and plan) — gate passed, awaiting reviewer audit
- Round: 1
- Last gate result: PASSED (exit 0, docs/evidence/M0.md "gate")
- Next action: wait for the reviewer's verdict on M0; do not start M1

## Milestones
| ID | Title | Status | Reviewer verdict | Evidence |
|---|---|---|---|---|
| M0 | Recon and plan | gate passed, awaiting review | - | docs/evidence/M0.md |
| M1 | Headless core and storage | not started | - | docs/evidence/M1.md |
| M2 | TUI shell and controls | not started | - | docs/evidence/M2.md |
| M3 | Shader-style effects | not started | - | docs/evidence/M3.md |
| M4 | Polish and handover | not started | - | docs/evidence/M4.md |

## Plan for the current assignment (10 lines max; rewrite each assignment)
- R1: fill docs/api-notes.md with one row per external fact needed by M1-M3, each with a real source.
- R4: verify ratatui 0.30.2, crossterm 0.29.0, directories 6.0.0, TestBackend, Terminal, init/restore and ProjectDirs in the local registry source; compile a throwaway probe in /tmp/opencode.
- R5: capture rustc, cargo, make, script, TERM, COLORTERM, tput, tty and pty size through scripts/ev.sh.
- R2/R3: write the M0-M4 table and seven-line plan in PROGRESS, and six questions with defaults in OPEN_QUESTIONS.
- R6: commit the template, intake docs and evidence on branch main with a message starting "M0:"; no Cargo.toml and no src/.
- R7: record sha256 of the six protected files before and after the commit and compare with docs/reviews/M0.md.
- R8: append the gate output, fill the evidence map and return one report line per requirement.

## Decisions (one line each, with the reason)
- Use ratatui 0.30 default features plus crossterm 0.29 and directories 6, because ratatui-crossterm defaults to crossterm 0.29 (verified in api-notes).
- Verify APIs by compiling a probe in /tmp/opencode/m0-probe, because the repo gate forbids app code and the local registry has the exact versions.
- Record the real high-score path as $XDG_DATA_HOME/2048/highscore with ~/.local/share fallback, verified by running ProjectDirs on this machine.
- Keep M0 free of .gitignore, Cargo.toml and src/ to respect the gate; M1 adds the Cargo project and Makefile.

## Known issues
- The agent shell is headless (no tty), so live neon rendering and the 100x36 look can only be checked by a human or by TestBackend snapshots.
- crates.io API returned HTTP 403, but the three required crates are already in the local registry cache at the fixed versions.
