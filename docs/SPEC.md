# SPEC — 2048

Working rules are in `AGENTS.md`. This file says **what** to build, in what order, and how success is judged. The reviewer fills it during intake. The coder reads it and never edits it.

Every intake placeholder must be replaced. Delete sections that do not apply instead of leaving them empty.

## 1. Goal
A single-player 2048 game for the project owner, played entirely from the keyboard in a terminal. It must feel polished: neon 24-bit colour, a persistent best score, and "shader-style" effects — a CPU plasma background, merge shockwaves and particle sparks — rendered through animated ANSI cells. "Working" means the standard 2048 rules are exact, every control works, the best score survives a restart, and the effects are visible in deterministic frame captures.

## 2. Non-goals
- No GPU, OpenGL, GLSL or image protocol. Shader-style means the same maths (plasma, radial waves, particles) evaluated per terminal cell and drawn as colour/style, not a GPU shader.
- No mouse, touch, network, accounts, online leaderboard or cloud sync. The named high-score table is a local file only (M8); it never contacts anything.
- No undo, board-size variants, themes, sound or mobile support.
- No Windows-specific verification in this project; the code should stay portable and Linux is the acceptance platform.

## 3. Environment
- Linux-first, portable Rust; terminal with 24-bit truecolour support (modern xterm, kitty, alacritty, foot, iTerm2).
- Target size **100 x 36** (width x height); minimum supported **60 x 30**. Order is always columns x rows. Below the minimum the app shows a readable notice and still quits cleanly.
- Animation target 60 FPS (configurable `--fps`); core rules are instant. The live loop draws, then waits until the next frame deadline instead of polling for a whole frame after drawing, so the achieved rate is not reduced by draw time.
- No network. Offline only. The high score is local.
- Input: arrow keys and WASD; R restart; Q/Esc quit; Space/Enter to continue after winning.

## 4. Layout / interface
```
┌ 2048 ──────────────────────────────────────────────────────────────────────────┐
│                                                                                │
│                                                                                │
│                                                                                │
│                                                                                │
│               ╭ SCORE 1234   BEST 5678 ──────────────────────────────╮          │
│               │ ╭───────╮ ╭───────╮ ╭───────╮ ╭───────╮             │          │
│               │ │       │ │       │ │       │ │       │             │          │
│               │ │   2   │ │       │ │       │ │       │             │          │
│               │ │       │ │       │ │       │ │       │             │          │
│               │ ╰───────╯ ╰───────╯ ╰───────╯ ╰───────╯             │          │
│               │  ... 4 rows of cards, 1 cell apart ...              │          │
│               ╰──────────────────────────────────────────────────────╯          │
│                                                                                │
│                                                                                │
│                                                                                │
│    Arrows / WASD move   R restart   Q quit                                     │
└────────────────────────────────────────────────────────────────────────────────┘
```
- Outer frame: one border around the whole terminal; the title `2048` sits on its top edge, left.
- Play-area frame: 41 x 25 cells (a 39 x 23 board plus one border cell on every side), centred with floor division. At 100x36 it starts at column 29, row 5; at 60x30 at column 9, row 2. It never overlaps the header row, the footer or the outer frame.
- Board: 4 x 4 rounded cards, each 9 cells wide x 5 rows tall, one blank cell between neighbours. The cards keep this fixed size on larger terminals; the extra space shows the plasma. Empty cells are dim outlines.
- Tile value: bold, on the middle inner row of its card (card row 2 of 5) and horizontally centred inside the 7-cell inner width with left padding `(7 - digits) / 2`.
- `SCORE` and `BEST`: on the play-area frame's top edge, `SCORE <n>` then three spaces then `BEST <n>`, starting two cells after the top-left corner; the rest of the edge is `─`. Both use the header style; the `BEST` segment pulses when a new record is set. The outer header shows the title only.
- The `SCORE` number counts up to its new value over 450 ms with an ease-out (section 5.7); `BEST` changes immediately. The count is display only and never changes the rules, the score or the RNG.
- Footer: key legend and a transient storage warning slot.
- Overlays: win (`2048!` + continue/restart), game over (final score + restart), and the M8 name-entry and high-score views below.
- High-score panel (M8): at terminals at least 100 columns x 30 rows, a 20 x 12 bordered block titled `HIGH SCORES` sits two blank columns right of the play-area frame, its top edge on the play area's top row (at 100x36: left 72, top 5). Its 10 content rows read `{name:<10} {score:>6}`, best first, left-aligned one cell inside the block; rows beyond the table stay blank. Terminals below 100 columns show no panel.
- Table overlay (M8): on terminals without the panel a centred 26 x 15 bordered block titled `HIGH SCORES` shows the same 10 rows; `H` toggles it. During play its bottom content row is `H close`; after game over its last two content rows are `SCORE <shown>` and `R restart   H close`, and the overlay is shown automatically (H switches back to the standard game-over overlay).
- Name-entry overlay (M8): a centred 34 x 7 bordered block titled `NEW HIGH SCORE`, shown instead of the game-over overlay when a qualifying score ends a game; its content rows are `SCORE <shown>`, blank, `NAME: <typed>_` (no `_` at 10 characters), blank, `ENTER save   ESC skip`.
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
- The in-game best is `max(loaded table top, current score)`; only named entries and the legacy migration persist (section 5.3).

### 5.2 Deterministic RNG (contract for reproducible seeds)
- Xorshift64*: `x ^= x >> 12; x ^= x << 25; x ^= x >> 27; state = x; output = x * 0x2545F4914F6CDD1D` (wrapping u64). A seed of 0 is replaced by `0x9E3779B97F4A7C15`.
- Spawn uses two draws in this order: `cell = next_u64() % empty_count` indexing the ascending list of empty cell indices, then `value = 4` when `next_u64() % 10 == 0`, else `2`.
- New games call spawn twice, in that order.

### 5.3 High-score table and storage (M8)
- Directory: `ProjectDirs::from("", "", "2048").data_dir()` (`$XDG_DATA_HOME/2048` on Linux).
- `highscores` (the table): one entry per line, `NAME SCORE`, sorted by score descending, at most 10 entries. A new score is inserted at its sorted position (ties go after the equal entries) and the list is truncated to 10.
- Names: 1-10 characters from `A-Z` and `0-9`, stored uppercase. The parser also accepts `-`, so a migrated legacy score can live as `---`.
- Best score: the score of the first entry, or 0 for an empty table.
- A score qualifies for the table when it is > 0 and either the table holds fewer than 10 entries or the score is greater than the 10th entry's score.
- Only named entries and the legacy migration are persisted. The in-game best and the new-best pulse follow section 5.1; quitting or restarting before game over saves nothing.
- A saved entry stores the real `game.score`, never the displayed count.
- Load: a missing or unreadable file is an empty table, never an error. A malformed line is skipped; scores parse as `u64`.
- Writes go to a sibling `.tmp` file and are renamed into place.
- If saving fails, the game keeps running, shows a warning in the footer for a few seconds and keeps the entry in memory; it never panics.
- `highscore` (legacy, one decimal `u64` and a newline): read once for migration only. When `highscores` is missing and `highscore` holds v > 0, the loaded table is the single entry `--- v`, written on the next save. `highscore` is never written again.
- `--demo`, `--demo-merge` and `--no-persist` use an in-memory store and never touch either real file.

### 5.4 Effects (all deterministic under fixed `dt`)
- **Plasma background**: per-cell colour from `sin`/`cos` of column, row and phase; phase advances by `dt`; dim enough to keep tiles readable.
- **Merge ripple** (M7; replaces the M3 shockwave): for every merge a wave leaves the merged cell and travels outward at **40 cells per second** for **900 ms** (`age < 900`), bouncing off the play-area walls and fading out. One wave per merge, each with its own source and merged value.
  - Space: the play area's 39 x 23 cells, `x` in `0..=38`, `y` in `0..=22`, `(0, 0)` top-left. The source is the merged card's centre `(col * 10 + 4.5, row * 6 + 2.5)`; distance is Euclidean with a row aspect of 2, `sqrt(dx*dx + (2*dy)*(2*dy))`, so one row counts as two columns and the crest reads round on a 1:2 terminal cell.
  - Walls: the play area's outer edges at `x = -0.5`, `x = 38.5`, `y = -0.5`, `y = 22.5`. A source at `(sx, sy)` has nine images: every combination of `sx`, `-1 - sx` (left wall) and `77 - sx` (right wall) with `sy`, `-1 - sy` (top wall) and `45 - sy` (bottom wall). Reflection is what makes the ripple return from a wall.
  - Wavefront: `radius = age_ms as f64 * 0.04` (no rounding). A cell is on the front for an image when `|distance - radius| <= 0.75`.
  - Amplitude: `1.0 - age_ms as f64 / 900.0`, multiplied by `0.55` for each wall crossed to reach that image (so `0.3025` for an image reflected off two walls). Each half-cell uses its strongest image (next bullet).
  - Rendering (M11): sample the front at the two half-cell centres `(x, y - 0.25)` and `(x, y + 0.25)`. A half is lit when its amplitude is `>= 0.10`; below that it is blank. Top half only: `▀` with fg = the top half's colour, bg = the background. Bottom half only: `▄` with fg = the bottom half's colour, bg = the background. Both: `▀` with fg = the top half's colour and bg = the bottom half's colour. The `▓`/`▒`/`░` glyph classes are no longer used by the ripple.
  - Brightness: a lit half's colour is the merged tile's colour mixed with the renderer's background by that half's amplitude — `fg = bg + (colour - bg) * amplitude` per RGB channel, rounded to the nearest integer. A reflection therefore starts at 55% brightness and every crest dims as it ages (`(13,2)` at 224 ms is `Rgb(1, 152, 197)` for a value-4 merge, at 480 ms `Rgb(3, 98, 130)`, at 700 ms `Rgb(5, 51, 72)`).
  - Layering: the ripple is drawn after the board cards and before the tile labels, so a digit always wins over a ripple glyph in the same cell.
- **Sparks**: 14 particles per merge, lifetime **500 ms**, characters `·`, `✦`, `+`, `*`, drifting outward with drag; drawn only inside the play area.
- **Tile flash**: merged tiles brighten for **180 ms**.
- **Spawn pop**: newly spawned tiles brighten for **120 ms**.
- **New best**: the `BEST` value pulses for **600 ms**.
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
| H | toggle the high-score table overlay (M8) on terminals without the panel; after game over it is shown automatically and H switches back to the game-over overlay. Where the panel is visible H does nothing. |
| any move key while game over | ignored |
| any move key while the win overlay is visible | ignored until dismissed |
| any move key while the table overlay is open during play | ignored; H closes it, R and Q still work |

Name entry (M8), shown after game over when the score qualifies for the table:
| Key | Action |
|---|---|
| A-Z, 0-9 | append one character, uppercased (maximum 10; further characters are ignored) |
| Backspace | delete the last character; no-op when empty |
| Enter | save the entry and close the entry overlay; with fewer than 1 character nothing happens |
| Esc | discard the entry and close the overlay |
| R, Q, arrows, H | typed as characters (R, Q, H) or ignored (arrows); they must not restart, quit or toggle anything while entry is active |

A typed name is stored and displayed uppercase. Name entry opens only on the game-over transition, never mid-game, and only for a qualifying score (section 5.3).

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
| `--self-test-tui` | enter the real TUI, draw 120 frames at the live pacing, restore, print `self-test-tui: 120 frames in <secs>s (<fps> fps)` on stdout and exit 0 |
| `--fps <n>` | live frame rate (default 60) |
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

### 5.7 Score count-up (display only)
- The panel and the overlays show a *displayed* score that chases the real `game.score`. It never feeds back into the rules, the score, the best score or the RNG.
- Constant `SCORE_COUNT_MILLIS = 450`. A score change takes `from` = the value currently displayed and `to` = the new real score, resets the animation clock to 0 and advances it by the injected `dt`:
  - `elapsed >= 450` -> displayed = `to`;
  - otherwise, with `x = elapsed / 450` and `progress = 1 - (1 - x)^3` in `f64`, displayed = `from + floor((to - from) * progress)`.
- The score change and the `dt` of the same `update` call both count: `update` applies the action first (it may raise the score), then advances the count. That is what pins the `--demo-merge` frames in G15.
- A score change during a running count re-targets from the currently displayed value to the new real score and restarts the clock. All merges of one move are one change (their summed delta).
- `Action::Restart` shows 0 immediately, with no counting. `best` is never counted: it jumps at once and pulses.
- Every score display reads the displayed value through one accessor: the play-area top edge and the game-over overlay.
- `--dump` prints the real `game.score`, never the displayed value.
- The count is pure state advanced only by `dt`; the core reads no wall clock.

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
| G14 | score change 0 -> 16, then `dt` steps of 50 ms | displayed 0, 4, 8, 11, 13, 14, 15, 15, 15, 16 at t = 0, 50, 100, 150, 200, 250, 300, 350, 400, 450 ms | hand-computed from 5.7: `floor(16 * (1 - (1 - t/450)^3))` |
| G15 | `--demo-merge` (score 0 -> 4), frames 4 / 14 / 24 / 30 = 64 / 224 / 384 / 480 ms | `SCORE 1`, `SCORE 3`, `SCORE 3`, `SCORE 4`; `BEST 4` at every frame | hand-computed from 5.7: `floor(4 * (1 - (1 - t/450)^3))` = 1.475, 3.493, 3.987, then the 450 ms clamp |
| G16 | insert `AAA 100`, `BBB 100`, `CCC 50`, then `DDD 200` into an empty table | rows `DDD 200`, `AAA 100`, `BBB 100`, `CCC 50`; ties keep insertion order; an 11th entry below the 10th is dropped | hand-computed from 5.3 |
| G17 | a full table whose 10th score is 50; game over with 0, 50 and 51 | 0 and 50 do not qualify; 51 does | hand-computed from 5.3 |
| G18 | ripple source at board cell 0 (play-area `(4.5, 2.5)`), 100 ms | `(7,1)` is on the front (`d = sqrt(2.5^2 + 3^2) = 3.905`); `(0,4)` is not (`d = sqrt(4.5^2 + 3^2) = 5.408`); a Chebyshev front would light `(0,4)` and miss `(7,1)` | hand-computed from 5.4 |
| G19 | ripple source at board cell 0, 100 ms | play cell `(5,5)` is top-half only: its top-half centre `(5, 4.75)` has `d = sqrt(0.5^2 + 4.5^2) = 4.528` (on), while the cell centre `(5,5)` has `d = sqrt(0.5^2 + 5^2) = 5.025` (off); the bottom half is off too | hand-computed from 5.4 |

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
- **M5 Prettier board and real 60 FPS.** 9x5 cards with the value on the true middle row, the play-area frame with `SCORE` and `BEST` on its top edge, default 60 FPS with deadline pacing, measured self-test line, `make e2e` captures moved to `docs/evidence/e2e/`. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --render-once --seed 42 --no-persist | grep -q "SCORE 0" && cargo run --quiet -- --render-once --seed 42 --no-persist | grep -q "BEST 0" && script -qec "./target/debug/game2048 --self-test-tui --seed 42 --no-persist --fps 60" /dev/null | grep -Eq "self-test-tui: 120 frames in [0-9]+\.[0-9][0-9]s \(([5-6][0-9]) fps\)"'` exits 0.
- **M6 Score count-up.** The displayed score chases the real score with the 450 ms ease-out of section 5.7: re-target from the shown value, restart resets it, `BEST` jumps at once, `--dump` stays real, and one accessor feeds the panel and the game-over overlay. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --render-once --demo-merge --frame 4 --no-persist | grep -q "SCORE 1" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "SCORE 3" && cargo run --quiet -- --render-once --demo-merge --frame 30 --no-persist | grep -q "SCORE 4" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "BEST 4" && cargo run --quiet -- --dump --seed 42 | diff -q - tests/fixtures/dump_seed42.txt'` exits 0.
- **M7 Merge ripple.** Replace the shockwave with the bouncing, fading ripple of section 5.4, keep sparks/flash/pop, update the effect snapshots and captures. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▓" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▒" && cargo run --quiet -- --dump --seed 42 | diff -q - tests/fixtures/dump_seed42.txt && make e2e'` exits 0.
- **M8 Arcade high-score table.** Named table (section 5.3), name entry after game over (section 5.5), score panel at 100+ columns and `H` overlay below, legacy `highscore` migration, named restart proof. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --dump --seed 42 --no-persist | diff -q - tests/fixtures/dump_seed42.txt && cargo run --quiet -- --render-once --seed 42 --no-persist | grep -q "HIGH SCORES" && cargo test --offline --test restart named_entry_survives_a_process_restart -- --exact && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▓" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▒" && make e2e'` exits 0.
- **M9 Handover refresh.** README, acceptance rows, clean-clone gate for M5-M8, no stale evidence pointers. *Gate:* `bash -o pipefail -c 'rm -rf /tmp/2048-clean && git clone . /tmp/2048-clean && cd /tmp/2048-clean && make check && make e2e && test -z "$(git status --porcelain)"'` exits 0; every acceptance row A1-A11 is complete in `docs/EVIDENCE.md`.
- **M10 Round merge ripple (post-handover).** Replace the Chebyshev (square) wavefront with the aspect-corrected Euclidean circle of section 5.4; keep the radius, 900 ms lifetime, ±0.75 window, nine wall images, 0.55 reflection factor and colour fade, and update the pinned cells, effect fixtures and demo-merge capture. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▓" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▒" && cargo run --quiet -- --dump --seed 42 | diff -q - tests/fixtures/dump_seed42.txt && make e2e'` exits 0.
- **M11 Half-block ripple (post-handover).** Draw the round front in half-block subcells (section 5.4): sample `y ± 0.25`, render `▀`/`▄` with two colours per cell, keep the metric/radius/lifetime/window/walls/0.55/fade, and regenerate the effect fixtures and demo-merge capture. *Gate:* `bash -o pipefail -c 'make check && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▀" && cargo run --quiet -- --render-once --demo-merge --frame 14 --no-persist | grep -q "▄" && cargo run --quiet -- --dump --seed 42 | diff -q - tests/fixtures/dump_seed42.txt && make e2e'` exits 0.

## 9. Acceptance checklist (IDs referenced in EVIDENCE.md)
- **A1** At 100x36 and 60x30 the UI renders without panic, overlap or truncation (snapshot tests + reported dimensions).
- **A2** All keys in section 5.5 are covered by input tests, including win/game-over blocking.
- **A3** The high-score table survives a process restart through the real file path (isolated `XDG_DATA_HOME` e2e); demo/no-persist never write it.
- **A4** Merge effects are visible and deterministic at the pinned frames after a merge; plasma, sparks and flash are present; effects do not alter rules or RNG.
- **A5** The terminal is restored after normal quit and after a simulated panic; `--self-test-tui` exits 0.
- **A6** `make check` and `make e2e` are green; every requirement has evidence.
- **A7** At 100x36 and 60x30 the play-area frame is exactly 41x25, the cards are 9x5 with one-cell gaps, and every tile value sits on its card's middle row, centred within one cell (M5).
- **A8** The live loop and the self-test pace to a 60 FPS deadline: `--self-test-tui --fps 60` prints a measured rate of 50-69 fps for 120 frames (M5).
- **A9** The displayed score follows the G14/G15 values, re-targets from the shown value, never exceeds the real score, resets on restart, and leaves `BEST` immediate while `--dump` prints the real score (M6).
- **A10** The merge ripple is visible, deterministic, round rather than square (aspect-corrected Euclidean), drawn in half-block subcells, reflects off at least one play-area wall and fades out before it expires (M7, round crest M10, subcell M11).
- **A11** A named score entered after game over is stored, reloaded and displayed in the table after a restart (M8).

## 10. Final report format
(1) Filled `docs/EVIDENCE.md`; (2) list of anything unverified or blocked, with reasons; (3) exact steps to run the game and where the high score is stored.
