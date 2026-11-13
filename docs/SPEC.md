# SPEC — 2048

Working rules are in `AGENTS.md`. This file says **what** to build, in what order, and how success is judged. The reviewer fills it during intake. The coder reads it and never edits it.

Every intake placeholder must be replaced. Delete sections that do not apply instead of leaving them empty.

## 1. Goal
A single-player 2048 game for the project owner, played entirely from the keyboard in a terminal. It must feel polished: neon 24-bit colour, a persistent best score, and "shader-style" effects — a CPU plasma background, merge shockwaves and particle sparks — rendered through animated ANSI cells. "Working" means the standard 2048 rules are exact, every control works, the best score survives a restart, and the effects are visible in deterministic frame captures.

## 2. Non-goals
- No GPU, OpenGL, GLSL or image protocol. Shader-style means the same maths (plasma, radial waves, particles) evaluated per terminal cell and drawn as colour/style, not a GPU shader.
- No mouse, touch, network, accounts, online leaderboard or cloud sync.
- No undo, board-size variants, themes, sound or mobile support.
- No Windows-specific verification in this project; the code should stay portable and Linux is the acceptance platform.

## 3. Environment
- Linux-first, portable Rust; terminal with 24-bit truecolour support (modern xterm, kitty, alacritty, foot, iTerm2).
- Target size **100 x 36** (width x height); minimum supported **60 x 30**. Order is always columns x rows. Below the minimum the app shows a readable notice and still quits cleanly.
- Animation target 30 FPS (configurable `--fps`); core rules are instant.
- No network. Offline only. The high score is local.
- Input: arrow keys and WASD; R restart; Q/Esc quit; Space/Enter to continue after winning.

## 4. Layout / interface
```
┌ 2048 ────────────────────────────────  SCORE 1234   BEST 5678 ┐
│                                                               │
│   ╭───────╮ ╭───────╮ ╭───────╮ ╭───────╮                     │
│   │   2   │ │   4   │ │       │ │       │                     │
│   ╰───────╯ ╰───────╯ ╰───────╯ ╰───────╯                     │
│   ...                                                         │
│                                                               │
│   Arrows / WASD move   R restart   Q quit                     │
└───────────────────────────────────────────────────────────────┘
```
- Header: title left, `SCORE` and `BEST` right; the best flashes when a new record is set.
- Board: 4 x 4 rounded cards, each 8 cells wide x 4 rows tall, centred; the cards keep this fixed size on terminals larger than 100x36 (the extra space shows the plasma). Empty cells are dim outlines.
- Footer: key legend and a transient storage warning slot.
- Overlays: win (`2048!` + continue/restart) and game over (final score + restart).
- Too-small terminal: a centred notice with the required size; no board rendering.

## 5. Behaviour
### 5.1 Rules (exact standard 2048)
- 4x4 board; start with two randomly placed tiles.
- A move slides every tile as far as possible in that direction.
- Two equal tiles that meet merge into their sum and add that sum to the score.
- A tile produced by a merge never merges again during the same move; each source tile participates in at most one merge.
- After a move that changes the board, exactly one new tile appears: value 2 with probability 9/10, otherwise 4.
- A move that changes nothing does not spawn and does not score.
- Reaching 2048 sets the win state once; the player may continue. Game over is when no empty cell or equal neighbour exists.
- R starts a new game and keeps the best score.
- Best score is `max(previous best, current score)`.

### 5.2 Deterministic RNG (contract for reproducible seeds)
- Xorshift64*: `x ^= x >> 12; x ^= x << 25; x ^= x >> 27; state = x; output = x * 0x2545F4914F6CDD1D` (wrapping u64). A seed of 0 is replaced by `0x9E3779B97F4A7C15`.
- Spawn uses two draws in this order: `cell = next_u64() % empty_count` indexing the ascending list of empty cell indices, then `value = 4` when `next_u64() % 10 == 0`, else `2`.
- New games call spawn twice, in that order.

### 5.3 High score storage
- Default path: `ProjectDirs::from("", "", "2048").data_dir()/highscore`, i.e. `$XDG_DATA_HOME/2048/highscore` on Linux.
- File contains one decimal `u64` and a newline. Missing or corrupt file loads as 0, never crashes.
- Writes go to a sibling `.tmp` file and are renamed into place.
- If saving fails, the game keeps running and shows a warning in the footer for a few frames; it never panics.
- `--demo`, `--demo-merge` and `--no-persist` use an in-memory store and never touch the real file.

### 5.4 Effects (all deterministic under fixed `dt`)
- **Plasma background**: per-cell colour from `sin`/`cos` of column, row and phase; phase advances by `dt`; dim enough to keep tiles readable.
- **Merge shockwave**: for every merge, a ring expands from the destination cell for **450 ms**. Characters `░`, `▒`, `▓` mark the ring; colour is derived from the merged tile value.
- **Sparks**: 14 particles per merge, lifetime **500 ms**, characters `·`, `✦`, `+`, `*`, drifting outward with drag; drawn only inside the board area.
- **Tile flash**: merged tiles brighten for **180 ms**.
- **Spawn pop**: newly spawned tiles brighten for **120 ms**.
- **New best**: the BEST value pulses for **600 ms**.
- Animations update from an injected `dt`; core logic and tests never read the wall clock. The `--frame N` contract uses exactly 16 ms per step (`elapsed = N * 16 ms`).
- Effects must never change the game state or the timing of the RNG.

### 5.5 Keys
| Key | Action |
|---|---|
| Left / A | move left |
| Right / D | move right |
| Up / W | move up |
| Down / S | move down |
| R | new game |
| Q / Esc | quit |
| Space / Enter | dismiss the win overlay ("keep playing") |
| any move key while game over | ignored |
| any move key while the win overlay is visible | ignored until dismissed |

### 5.6 CLI
| Flag | Meaning |
|---|---|
| `--seed <u64>` | deterministic RNG seed (default: system time) |
| `--moves <lrud>` | apply moves to a headless game (with `--dump`) |
| `--dump` | print the core result as text and exit; no terminal setup |
| `--render-once` | print one plain-text frame via `TestBackend` and exit |
| `--frame <n>` | fixed animation frame for `--render-once` (16 ms steps) |
| `--demo` | seeded showcase board, in-memory score, some effects triggered |
| `--demo-merge` | demo board plus an immediate merge effect for captures |
| `--no-persist` | force the in-memory high-score store |
| `--self-test-tui` | enter the real TUI, draw a few frames, restore and exit 0 |
| `--fps <n>` | live frame rate (default 30) |
| `--help` | usage |

`--dump --seed 42` (no moves) must print exactly:
```
board:
2 0 0 0
0 0 0 0
0 0 0 0
2 0 0 0
score: 0
best: 0
won: false
game_over: false
```

## 6. Golden examples (exact input -> output pairs)
Source: standard 2048 rules, hand-computed below. Tests assert these exactly.

| ID | Input | Expected output | Source |
|---|---|---|---|
| G1 | row `2 2 0 0`, move left | row `4 0 0 0`; score +4; 1 merge at cell 0 value 4 | hand-computed |
| G2 | row `2 2 2 2`, move left | row `4 4 0 0`; score +8; 2 merges | hand-computed |
| G3 | row `4 4 8 0`, move left | row `8 8 0 0`; score +8; 1 merge (the new 8 does not chain) | hand-computed |
| G4 | row `2 0 2 4`, move left | row `4 4 0 0`; score +4; 1 merge at cell 0 | hand-computed |
| G5 | row `2 2 4 4`, move left | row `4 8 0 0`; score +12; 2 merges | hand-computed |
| G6 | row `2 4 2 4`, move left | unchanged; `moved=false`; score +0; no spawn | hand-computed |
| G7 | row `2 2 0 0`, move right | row `0 0 0 4`; merge at cell 3 value 4 | hand-computed |
| G8 | column `2 2 0 0` (cells 0,4), move up | cell 0 = 4, cell 4 = 0; merge at cell 0 | hand-computed |
| G9 | seed 42, two spawns from empty | first at cell 0 = 2, second at cell 12 = 2 | hand-computed from 5.2 |
| G10 | row `1024 1024 0 0`, move left | cell 0 = 2048; `won=true`; overlay hides after continue | hand-computed |
| G11 | full board with no equal neighbours | `game_over=true`; any move is a no-op | hand-computed |
| G12 | save 4096 then load from temp file | load = 4096; overwrite 8192 -> load = 8192 | hand-computed |
| G13 | missing or `"oops"` high-score file | load = 0, no panic | hand-computed |

## 7. Data sources and modes
| Source | What it provides | Purpose |
|---|---|---|
| `demo` | Seeded board, synthetic effects, in-memory score | Default screenshots, demos, `make demo` |
| `live` | Real keyboard play, real high-score file | Actual use |
| `test` | Temp files and `TestBackend` snapshots | CI, `make check` |

Fault injection flags: `--no-persist` (storage bypass), `--seed` (deterministic board), `--frame` (deterministic animation). Demo data never touches the real high-score path. Tests set a temp `XDG_DATA_HOME` or pass an explicit path.

Reference implementations to read before designing anything:
- Original 2048 by Gabriele Cirulli: `https://github.com/gabrielecirulli/2048` — rules and merge-once behaviour (VERIFY in M0).
- `ratatui` 0.30 examples/docs: `TestBackend`, `Terminal::draw`, `init/restore`, truecolour styles (VERIFY in M0).

## 8. Milestones and gates
Every gate is a runnable command with an expected result. After each gate the coder stops and the reviewer audits. Proof goes in `docs/evidence/M<n>.md`.

- **M0 Recon and plan (no app code).** Read the references. Fill `docs/api-notes.md` (facts tagged VERIFIED/UNVERIFIED with sources), `docs/PROGRESS.md` (milestone table and plan) and `docs/OPEN_QUESTIONS.md` (each question with a proposed default). List every file, credential or fact needed from the human. *Gate:* `test -s AGENTS.md && test -s docs/SPEC.md && test -s docs/api-notes.md && test -s docs/PROGRESS.md && test -s docs/OPEN_QUESTIONS.md` exits 0; all three coder docs have rows; `git log -1 --format=%h` returns a commit.
- **M1 Headless core and storage.** Cargo project, Makefile, pure board rules, deterministic RNG, high-score trait + file/memory stores, CLI dump. *Gate:* `make check` green (test count recorded); `cargo run -- --dump --seed 42` prints the exact block in section 5.6.
- **M2 TUI shell and controls.** ratatui/crossterm app, layout at 100x36 and 60x30, header/footer/overlays, all key bindings, terminal restore on quit/panic, `--render-once` and `--self-test-tui`. *Gate:* `make check` green including snapshots; `cargo run -- --render-once --seed 42 --no-persist` exits 0 and contains `2048`, `SCORE`, `BEST`; `script -qec 'target/debug/game2048 --self-test-tui --seed 42 --no-persist' /dev/null` exits 0.
- **M3 Shader-style effects.** Plasma background, merge shockwave, sparks, tile flash, spawn pop, new-best pulse, fixed-step frame contract, effect snapshot frames at 100x36. *Gate:* `make check` green including effect snapshots at frames 4/14/24; `cargo run -- --render-once --demo-merge --frame 14 --no-persist` exits 0 and shows merge effect characters; `make e2e` writes the frame captures.
- **M4 Polish and handover.** README, clean-clone run, full e2e, acceptance evidence. *Gate:* `make check` and `make e2e` green on a clean clone (`git clone . /tmp/2048-clean && cd /tmp/2048-clean && make check && make e2e`); `docs/EVIDENCE.md` has a row for every acceptance ID.

## 9. Acceptance checklist (IDs referenced in EVIDENCE.md)
- **A1** At 100x36 and 60x30 the UI renders without panic, overlap or truncation (snapshot tests + reported dimensions).
- **A2** All keys in section 5.5 are covered by input tests, including win/game-over blocking.
- **A3** High score survives a process restart through the real file path (isolated `XDG_DATA_HOME` e2e); demo/no-persist never write it.
- **A4** Merge effects are visible and deterministic at frames 4/14/24 after a merge; plasma, sparks and flash are present; effects do not alter rules or RNG.
- **A5** The terminal is restored after normal quit and after a simulated panic; `--self-test-tui` exits 0.
- **A6** `make check` and `make e2e` are green; every requirement has evidence.

## 10. Final report format
(1) Filled `docs/EVIDENCE.md`; (2) list of anything unverified or blocked, with reasons; (3) exact steps to run the game and where the high score is stored.
