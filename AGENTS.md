# AGENTS.md — 2048

Shared rules for every agent in this repo. opencode loads this file automatically. Follow every line. No intake placeholder may remain when work starts.

## The five rules that matter most
1. **No claim without proof.** Say "done", "passing" or "works" only if you ran the command in this session and the real output is in `docs/evidence/M<n>.md`.
2. **No guessing.** Never invent an external API, CLI flag, file format or library function. Look it up (docs, source, `--help`, compiler) or record it as UNVERIFIED in `docs/api-notes.md`.
3. **Stay in scope.** Do only the assigned milestone and requirements. No extra features, no rewrites of working code, no new dependencies.
4. **Stop at the gate.** Report, then wait. Never start the next milestone.
5. **No secrets.** Never print, log, commit or paste credentials or personal data.

## Mission
A polished, keyboard-first 2048 game that runs in a terminal for the project owner: neon 24-bit colour, a persistent high score, and shader-style merge effects rendered as animated ANSI cells. What to build: `docs/SPEC.md`. How to work: this file.

## Roles and ownership
- **Human** talks only to the reviewer. Holds secrets, accounts, hardware and final approval.
- **Reviewer** (primary agent) plans milestones, writes requirements, delegates, audits, gives verdicts. Never writes application code.
- **Coder** (subagent) implements one assignment and returns one report. No memory between assignments; the files below are its memory.

| File | Written by | Purpose |
|---|---|---|
| `AGENTS.md`, `docs/SPEC.md` | reviewer | Rules and spec. The coder never edits them. |
| `docs/reviews/M<n>.md` | reviewer | Requirements, gate, audit, verdict. |
| `docs/PROGRESS.md` | coder | Plan and status. |
| `docs/EVIDENCE.md` | coder | Requirement ID -> test -> command -> result. |
| `docs/evidence/M<n>.md` | coder via `scripts/ev.sh` | Raw proof. |
| `docs/OPEN_QUESTIONS.md`, `docs/api-notes.md` | coder | Questions with defaults; external facts. |

## Read order at the start of every assignment (coder)
1. `docs/reviews/M<n>.md`  2. `docs/SPEC.md` (sections 1-7 and your milestone in section 8)  3. `docs/PROGRESS.md`  4. `docs/api-notes.md`

## Fixed stack
- Rust 1.88+ (edition 2021), `ratatui` 0.30 with the `crossterm` backend, `crossterm` 0.29, `directories` 6 for the high-score path.
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `ratatui::backend::TestBackend` for headless UI snapshots, GNU `make`.
- The "shader" effects are CPU-rendered ANSI truecolour animations (plasma background, merge shockwave, particle sparks). There is no GPU or GLSL in this project; do not add one.
- Do not swap the stack or add a dependency without asking the reviewer.
- **Never write library APIs from memory.** Add the dependency with the package manager, read its docs/source, and let the compiler or test runner judge. Commit the lockfile.

## Architecture rules
1. **Pure core, thin shell.** Game rules, RNG, score and persistence contracts live in a library. `main` only parses flags and starts the shell.
2. **External systems behind interfaces.** High-score storage has file and in-memory implementations. The RNG is a trait with one deterministic implementation. Nothing above the interface knows which is active.
3. **Injectable time.** The core has no clock. Animation state advances by a `dt` supplied by the shell; tests pass fixed steps and assert exact frames.
4. **One module owns each invariant.** Board scoring/merging only in `game`; the high-score file format only in `highscore`; layout only in the renderer.
5. **Safe by default.** Demo, seeds and no-persist modes never touch the real high score. Tests never read the real HOME or XDG data directory.
6. **Mock/demo data never shares storage with real data.** `--demo` and `--no-persist` force an in-memory store.

### Profile: terminal UI
- Events -> actions -> `update(state, dt)` -> `render(state)`. `update` has no I/O; `render` only draws.
- Raw mode and the alternate screen are always restored, including after a panic (panic hook + RAII guard).
- No stdout/stderr printing while the TUI runs; write diagnostics to a log file or show them in the UI. `--dump` and `--render-once` are the only non-TUI modes.
- Draw with `ratatui` buffers so snapshots are deterministic; never depend on wall-clock time inside rendering.
- The smallest supported terminal is 60x30 (width x height). Below that show a readable notice, never a panic.

### Profile: Rust
- No `unwrap`/`expect` outside tests and `main`.
- Fragile external code (terminal setup, file storage) is quarantined behind one module with typed errors.

## Commands (create the `Makefile` in Milestone 1)
| Target | Runs |
|---|---|
| `make check` | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |
| `make run` | the real game: `cargo run --release` |
| `make demo` | seeded demo board and effects with `--demo --seed 42 --no-persist` |
| `make e2e` | deterministic headless frames and a persistence round-trip, written under `docs/evidence/` |

`make live-smoke` does not apply: there is no network service.

## Testing rules
Every behaviour in the spec has at least one automated test. Use the layers the spec needs:
1. **Unit** tests for the pure rules (slide, merge-once, score, spawn, win, game over).
2. **Golden** tests from the spec's exact input -> output pairs, asserted exactly.
3. **Snapshot** tests at 100x36 and 60x30 with `TestBackend`; review them by eye, never auto-accept.
4. **Input tests** mapping every key, including boundaries and the win/game-over overlays.
5. **Fake-time timelines** for animations: fixed `dt` steps, expected frames computed from the spec, never from the renderer's own output.
6. **Storage tests** with temporary paths only; corrupt, missing and round-trip cases.
7. **End-to-end** `--render-once`/frames against the real binary and an isolated `XDG_DATA_HOME`.
8. **Isolation guard:** tests never read the real HOME, config or high-score path.

A bug fix starts with a failing test that reproduces it.

## Evidence rules
- Run every command you will cite with `bash scripts/ev.sh M<n> "<label>" -- <command>`. It appends the command, UTC time, real output and exit code to `docs/evidence/M<n>.md`. Never type, paste or edit output blocks by hand; hand-written output is treated as fabricated.
- Captures and snapshots go in `docs/evidence/M<n>/` and are referenced from the evidence file.
- If you could not run something, say so and list it as unverified. Never fabricate output, screenshots or fixtures. Label synthetic data as synthetic.
- At the end of each milestone update `docs/EVIDENCE.md`: ID, test names, command, result, date.
- A milestone is finished only when its Gate passes and the proof is in the evidence file.
- Answer every numbered requirement from the reviewer with: done / not done, test name, proof. Do not substitute a different test for the one requested; if you think another is better, say so and still provide the requested one.
- Never edit a test to make it pass. If you believe the test is wrong, say so in the report.

## Working loop
1. Write a plan of at most 10 lines to `docs/PROGRESS.md`.
2. Small steps: failing test, make it pass, refactor, commit (`M<n>: <what>`). Never push.
3. Run `make check` after every step once the Makefile exists. Do not silence warnings without a written reason.
4. At the end: run the Gate through `scripts/ev.sh`, update `docs/EVIDENCE.md` and `docs/PROGRESS.md`, send the final report (format in `.opencode/agents/coder.md`), stop.

## Hard don'ts
- Do not hardcode sizes or paths that should adapt or be configured.
- Do not commit secrets or personal data.
- Do not expand scope or rewrite working modules while implementing something else.
- Do not hide problems: disclose deviations and near-misses (for example a test that could have touched real data).

## Stop and report BLOCKED when
- You need credentials, captures or facts only the human has.
- The same failure survives three honest attempts (write it up in `docs/OPEN_QUESTIONS.md`).
- A spec item is ambiguous or two items conflict.
- You want a new dependency or a stack change.

BLOCKED means: stop and return the final report with `STATUS: BLOCKED`, the exact question and a proposed default. The reviewer relays it to the human.

## Reminder (the top rules again)
No claim without proof. No guessing. Stay in scope. Stop at the gate. No secrets.
