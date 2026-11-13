# OPEN QUESTIONS — closed decisions

> Owned by the coder. All questions from M0-M11 are closed; the table keeps one
> compact row per decision so a future agent knows why the code looks as it does.
> No row blocks work. The reviewer can add new questions to a new
> `docs/reviews/M<n>.md`.

| # | Decision | Outcome |
|---|---|---|
| Q1 | Live colour and animation feel cannot be judged from the headless agent shell. | Accepted live by the user on 2026-11-13 ("go, everything looks great."); TestBackend snapshots remain the recorded fallback proof. |
| Q2 | `XDG_DATA_HOME` may be unset or relative. | Live play uses `ProjectDirs::from("", "", "2048")`; every test and e2e run sets an absolute temporary `XDG_DATA_HOME`. |
| Q3 | The crates.io API was blocked; only cached crates were guaranteed. | No dependency was added; the fixed stack resolves offline from the local registry. |
| Q4 | SPEC did not pin the `--demo-merge` board, merge cell or start frame. | Seed-42 demo board, one merge at cell 0, effects visible at frames 4/14/24. |
| Q5 | When is the best score written, and what if the clean-quit save fails? | Superseded by M8: only named entries persist and quitting saves nothing; a failed save still warns once and keeps the entry in memory. |
| Q6 | Do R and Q work while the win overlay is visible? | Yes: only move keys are ignored until Space/Enter dismisses the overlay. |
| Q7 | SPEC's seed-42 dump example contradicted §5.2/G9 on the second tile. | Human-approved: the second tile is cell 12; §5.6 and the fixture were corrected, the RNG stayed as specified. |
| Q8 | How does the live loop advance effects without a clock in the core? | The shell measures elapsed time and passes `dt`, clamped to 250 ms; pure modules receive only the injected value. |
| Q10 | The play-area edge sketch and the `panel.left + 2` rule differed. | Implemented as reviewed: `─` at `+1`, `SCORE` starts at `+2`; corner cells stay intact. |
| Q11 | The count-up feel needed a live human check. | Accepted live 2026-11-13; `SCORE_COUNT_MILLIS = 450` with the cubic ease-out stays. |
| Q12 | Two test merges at cells 0/5 cannot come from one real move, and the label layer paints a whole row. | Keep the effects-layer test for 0/5 and cover the wiring with a real Left move (cells 0/12); the label layer stays as approved in M5 because no pin sits on a label row. |
| Q14 | The M10 reflected pin `(3,2)` can never render due to the value-4 label and a spark. | Effect layer keeps `front_at(3,2) = ('▒', 0.413111)`; rendered reflected pins use `(2,5)` (`▒`, `Rgb(4,87,117)`). |
| Q15 | M11 removed `▓`/`▒`/`░`, but a protected CLI test asserted them; two colour literals needed impossible amplitudes. | Reviewer allowed exactly two assertion lines in `tests/cli.rs` (`▀`/`▄`, `✦` kept) and accepted the §5.4-derived `(0,4)` values `Rgb(3,96,128)`/`Rgb(1,168,216)`. |
