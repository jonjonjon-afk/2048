//! The TUI shell: storage wiring, the live event loop and the pty self-test.
//!
//! The shell is the only place where the pure application state meets the
//! filesystem and the terminal. It turns key events into actions, applies the
//! returned effects to the high-score table, and draws the result.

use std::fmt;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};

use crate::action::Action;
use crate::game::Game;
use crate::highscore::HighScoreStore;
use crate::input::{action_for_key, entry_action_for_key};
use crate::render::{render, PANEL_MIN_WIDTH};
use crate::terminal::TerminalGuard;
use crate::update::{App, Effect};

/// Longest `dt` the live shell forwards; a resumed terminal must not jump the effects.
pub const MAX_DT_MILLIS: u64 = 250;

/// Frames the pty self-test draws before it reports the achieved rate.
pub const SELF_TEST_FRAMES: u32 = 120;

/// The next frame deadline.
///
/// `deadline` is the deadline the loop just finished; normally the next one is
/// one `frame` later, which keeps the fixed grid and absorbs draw time. When
/// `now` has already passed that grid point a whole frame was missed, so the
/// grid restarts from `now` — skipped frames, never a catch-up spiral.
pub fn next_frame_deadline(deadline: Instant, now: Instant, frame: Duration) -> Instant {
    let next = deadline + frame;
    if now < next {
        next
    } else {
        now + frame
    }
}

/// What one `--self-test-tui` run measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelfTestReport {
    /// Frames drawn.
    pub frames: u32,
    /// Wall-clock time the run took.
    pub elapsed: Duration,
}

impl SelfTestReport {
    /// The achieved frame rate, rounded to a whole frame per second.
    pub fn fps(&self) -> u32 {
        let seconds = self.elapsed.as_secs_f64();
        if seconds > 0.0 {
            (f64::from(self.frames) / seconds).round() as u32
        } else {
            0
        }
    }
}

impl fmt::Display for SelfTestReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "self-test-tui: {} frames in {:.2}s ({} fps)",
            self.frames,
            self.elapsed.as_secs_f64(),
            self.fps()
        )
    }
}

/// Application state plus the store that owns the high-score table.
pub struct Shell<S: HighScoreStore> {
    pub app: App,
    store: S,
    /// Stderr lines queued for `main`; written only after the terminal is restored.
    warnings: Vec<String>,
}

/// What an interactive session leaves for the binary entry point to print.
#[derive(Clone, Debug, Default)]
pub struct SessionOutcome {
    warnings: Vec<String>,
}

impl SessionOutcome {
    /// Warnings to write to stderr after the terminal guard has restored the screen.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Consume the outcome and return the queued warning lines.
    pub fn into_warnings(self) -> Vec<String> {
        self.warnings
    }
}

impl<S: HighScoreStore> Shell<S> {
    /// Load the table, set the in-game best from its top and start a new game.
    pub fn start(store: S, seed: u64) -> Self {
        let table = store.load();
        let best = table.best();
        Self::from_app(App::with_table(Game::new(seed, best), table), store)
    }

    /// Wrap an existing application state; used by tests and previews.
    pub fn from_app(app: App, store: S) -> Self {
        Self {
            app,
            store,
            warnings: Vec::new(),
        }
    }

    /// The store behind this shell, for assertions and diagnostics.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Warning lines queued so far. The shell never prints; `main` does.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Apply one action and perform the effects the update asked for.
    pub fn apply(&mut self, action: Option<Action>, dt: Duration) {
        for effect in self.app.update(action, dt) {
            match effect {
                Effect::SaveScores(table) => {
                    if let Err(error) = self.store.save(&table) {
                        // One line per session: a failed save must not spam stderr.
                        if self.warnings.is_empty() {
                            self.warnings.push(format!("warning: {error}"));
                        }
                        self.app.report_save_failure();
                    }
                }
            }
        }
    }
}

/// Run the interactive game until the player quits.
pub fn run_live<S: HighScoreStore>(store: S, seed: u64, fps: u32) -> io::Result<SessionOutcome> {
    run_loop(Shell::start(store, seed), fps)
}

/// Run the interactive game from an already-built application state (demo mode).
pub fn run_live_app<S: HighScoreStore>(app: App, store: S, fps: u32) -> io::Result<SessionOutcome> {
    run_loop(Shell::from_app(app, store), fps)
}

fn run_loop<S: HighScoreStore>(mut shell: Shell<S>, fps: u32) -> io::Result<SessionOutcome> {
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard::new(ratatui::restore);
    let frame = Duration::from_secs(1) / fps.max(1);
    let mut deadline = Instant::now();
    let mut last = deadline;

    while !shell.app.quit {
        let area = terminal.draw(|area| render(area, &shell.app))?.area;
        // The draw knows the real terminal width, so the pure state can apply
        // the "H does nothing where the panel is visible" rule (SPEC §5.5).
        shell.app.set_panel_visible(area.width >= PANEL_MIN_WIDTH);

        // Wait for input only up to the next frame deadline. The draw happens
        // at the start of the next iteration, so draw time is part of the
        // frame period instead of being added to it.
        let remaining = deadline.saturating_duration_since(Instant::now());
        let action = if event::poll(remaining)? {
            match event::read()? {
                Event::Key(key) => {
                    // Name entry owns the keyboard while it is open (M8).
                    if shell.app.entry_active() {
                        entry_action_for_key(key)
                    } else {
                        action_for_key(key)
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        let now = Instant::now();
        deadline = next_frame_deadline(deadline, now, frame);

        // Measured, clamped dt: animation speed follows real time, but a long
        // stall (sleep, suspend) may not skip whole effects.
        let elapsed = now
            .duration_since(last)
            .min(Duration::from_millis(MAX_DT_MILLIS));
        last = now;
        shell.apply(action, elapsed);
    }

    // `_guard` is dropped here, before the caller can print the queued warning.
    Ok(SessionOutcome {
        warnings: shell.warnings,
    })
}

/// Enter the real terminal, draw 120 paced frames, restore and report.
///
/// This is the automated proof that raw mode and the alternate screen are set
/// up and restored under a pty, and that the live pacing reaches the requested
/// frame rate. It never reads input and never persists; `main` prints the
/// returned report only after the terminal guard has been dropped.
pub fn run_self_test<S: HighScoreStore>(
    store: S,
    seed: u64,
    fps: u32,
) -> io::Result<SelfTestReport> {
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard::new(ratatui::restore);
    let shell = Shell::start(store, seed);
    let frame = Duration::from_secs(1) / fps.max(1);
    let start = Instant::now();
    let mut deadline = start;

    for _ in 0..SELF_TEST_FRAMES {
        terminal.draw(|area| render(area, &shell.app))?;

        let next = next_frame_deadline(deadline, Instant::now(), frame);
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
        deadline = next;
    }

    Ok(SelfTestReport {
        frames: SELF_TEST_FRAMES,
        elapsed: start.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::game::{Board, Direction};
    use crate::highscore::{FileStore, HighScoreError, HighScores, FILE_NAME};
    use crate::render::frame_text;

    #[derive(Debug)]
    struct RecordingStore {
        table: HighScores,
        saves: Vec<HighScores>,
        fail: bool,
    }

    impl RecordingStore {
        fn new(table: HighScores) -> Self {
            Self {
                table,
                saves: Vec::new(),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                table: HighScores::new(),
                saves: Vec::new(),
                fail: true,
            }
        }
    }

    impl HighScoreStore for RecordingStore {
        fn load(&self) -> HighScores {
            self.table.clone()
        }

        fn save(&mut self, table: &HighScores) -> Result<(), HighScoreError> {
            if self.fail {
                return Err(HighScoreError::NoDefaultPath);
            }
            self.saves.push(table.clone());
            self.table = table.clone();
            Ok(())
        }
    }

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    /// An explicit temp directory; no test reads the real HOME or data path.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!(
                "game2048-shell-{}-{}-{}-{}",
                std::process::id(),
                nanos,
                id,
                label
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn high_scores(rows: &[(&str, u64)]) -> HighScores {
        let mut table = HighScores::new();
        for (name, score) in rows {
            table.insert(*name, *score);
        }
        table
    }

    fn pair_app() -> App {
        let board =
            Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        App::new(Game::from_state(board, 0, 0, 42))
    }

    /// A full board with one merge available; the post-move spawn ends the game.
    fn transition_app(score: u64) -> App {
        let board = Board::from_rows([
            [2, 2, 8, 16],
            [16, 32, 64, 128],
            [8, 16, 32, 64],
            [64, 128, 8, 32],
        ]);
        App::with_table(Game::from_state(board, score, 0, 42), HighScores::new())
    }

    #[test]
    fn shell_loads_the_table_top_as_best() {
        let table = high_scores(&[("ALICE", 4096)]);
        let shell = Shell::start(RecordingStore::new(table.clone()), 42);

        assert_eq!(shell.app.game.best, 4096);
        assert_eq!(shell.app.table, table);
        assert!(shell.store().saves.is_empty());
    }

    #[test]
    fn quit_does_not_save_an_unnamed_score() {
        let game = Game::from_state(Board::empty(), 1234, 0, 42);
        let mut recording = Shell::from_app(App::new(game), RecordingStore::new(HighScores::new()));
        recording.apply(Some(Action::Quit), Duration::ZERO);
        assert!(recording.app.quit);
        assert!(
            recording.store().saves.is_empty(),
            "an unnamed quit must write nothing"
        );

        // The real file is unchanged too.
        let dir = TempDir::new("quit");
        let path = dir.path.join(FILE_NAME);
        fs::write(&path, "ALICE 4096\n").expect("write the table");
        let mut shell = Shell::start(FileStore::new(&path), 42);
        shell.apply(Some(Action::Quit), Duration::ZERO);
        assert!(shell.app.quit);
        assert_eq!(
            fs::read_to_string(&path).expect("read the table"),
            "ALICE 4096\n"
        );
        assert!(
            !FileStore::new(&path).temp_path().exists(),
            "no .tmp may be left behind"
        );
    }

    #[test]
    fn a_new_best_pulses_without_saving() {
        let mut shell = Shell::from_app(pair_app(), RecordingStore::new(HighScores::new()));

        shell.apply(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(shell.app.game.score, 2048);
        assert_eq!(shell.app.game.best, 2048);
        assert_eq!(shell.app.effects.best_pulse_age(), Some(Duration::ZERO));
        assert!(
            shell.store().saves.is_empty(),
            "only a named entry is persisted"
        );
    }

    #[test]
    fn save_failure_keeps_the_entry_and_warns() {
        let mut shell = Shell::from_app(transition_app(12), RecordingStore::failing());
        shell.apply(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert!(shell.app.entry_active(), "the game-over move opens entry");

        shell.apply(Some(Action::EntryChar('A')), Duration::ZERO);
        shell.apply(Some(Action::EntryEnter), Duration::ZERO);

        assert!(!shell.app.entry_active(), "Enter closes the entry");
        assert!(
            shell.store().saves.is_empty(),
            "the failing store records no save"
        );
        assert_eq!(shell.app.table.entries().len(), 1);
        assert_eq!(shell.app.table.best(), 16);
        assert!(shell.app.warning_active(), "the footer warning is live");
        assert_eq!(shell.warnings().len(), 1);
        assert!(
            shell.warnings()[0].starts_with("warning: "),
            "warning must be prefixed for stderr: {}",
            shell.warnings()[0]
        );
        assert!(!shell.app.quit, "a failed save must not trap the player");

        shell.apply(None, Duration::from_millis(16));
        assert!(!shell.app.quit, "the session continues after the failure");
    }

    #[test]
    fn failed_save_shows_footer_warning_for_three_seconds() {
        let mut shell = Shell::from_app(transition_app(12), RecordingStore::failing());
        shell.apply(Some(Action::Move(Direction::Left)), Duration::ZERO);
        shell.apply(Some(Action::EntryChar('A')), Duration::ZERO);
        shell.apply(Some(Action::EntryEnter), Duration::ZERO);

        let at_zero = frame_text(&shell.app, 100, 36).to_lowercase();
        assert!(at_zero.contains("not saved"), "warning missing at t = 0");

        let frame = Duration::from_millis(1000 / 30);
        for _ in 0..90 {
            shell.apply(None, frame);
        }

        let after_three_seconds = frame_text(&shell.app, 100, 36).to_lowercase();
        assert!(
            !after_three_seconds.contains("not saved"),
            "warning must be gone after 3000 ms"
        );
    }

    #[test]
    fn frame_pacing_skips_missed_frames() {
        let deadline = Instant::now();
        let frame = Duration::from_millis(16);

        assert_eq!(
            next_frame_deadline(deadline, deadline + Duration::from_millis(5), frame),
            deadline + Duration::from_millis(16),
            "an early frame keeps the fixed grid"
        );
        assert_eq!(
            next_frame_deadline(deadline, deadline + Duration::from_millis(16), frame),
            deadline + Duration::from_millis(32),
            "a frame that lands exactly on the deadline advances one period"
        );
        assert_eq!(
            next_frame_deadline(deadline, deadline + Duration::from_millis(20), frame),
            deadline + Duration::from_millis(36),
            "a missed frame restarts the grid from now, with no catch-up spiral"
        );
    }

    #[test]
    fn self_test_report_line_format() {
        let report = SelfTestReport {
            frames: 120,
            elapsed: Duration::from_millis(2000),
        };

        assert_eq!(SELF_TEST_FRAMES, 120);
        assert_eq!(
            report.to_string(),
            "self-test-tui: 120 frames in 2.00s (60 fps)"
        );

        let slower = SelfTestReport {
            frames: 120,
            elapsed: Duration::from_millis(2010),
        };
        assert_eq!(
            slower.to_string(),
            "self-test-tui: 120 frames in 2.01s (60 fps)"
        );
    }
}
