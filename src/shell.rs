//! The TUI shell: storage wiring, the live event loop and the pty self-test.
//!
//! The shell is the only place where the pure application state meets the
//! filesystem and the terminal. It turns key events into actions, applies the
//! returned effects to the high-score store, and draws the result.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};

use crate::action::Action;
use crate::game::Game;
use crate::highscore::HighScoreStore;
use crate::input::action_for_key;
use crate::render::render;
use crate::terminal::TerminalGuard;
use crate::update::{App, Effect};

/// Longest `dt` the live shell forwards; a resumed terminal must not jump the effects.
pub const MAX_DT_MILLIS: u64 = 250;

/// Application state plus the store that owns the best score.
pub struct Shell<S: HighScoreStore> {
    pub app: App,
    store: S,
}

impl<S: HighScoreStore> Shell<S> {
    /// Load the best score from the store and start a new game.
    pub fn start(store: S, seed: u64) -> Self {
        let best = store.load();
        Self::from_app(App::new(Game::new(seed, best)), store)
    }

    /// Wrap an existing application state; used by tests and previews.
    pub fn from_app(app: App, store: S) -> Self {
        Self { app, store }
    }

    /// The store behind this shell, for assertions and diagnostics.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Apply one action and perform the effects the update asked for.
    pub fn apply(&mut self, action: Option<Action>, dt: Duration) {
        for effect in self.app.update(action, dt) {
            match effect {
                Effect::SaveBest(score) => {
                    if self.store.save(score).is_err() {
                        self.app.report_save_failure();
                    }
                }
            }
        }
    }
}

/// Run the interactive game until the player quits.
pub fn run_live<S: HighScoreStore>(store: S, seed: u64, fps: u32) -> io::Result<()> {
    run_loop(Shell::start(store, seed), fps)
}

/// Run the interactive game from an already-built application state (demo mode).
pub fn run_live_app<S: HighScoreStore>(app: App, store: S, fps: u32) -> io::Result<()> {
    run_loop(Shell::from_app(app, store), fps)
}

fn run_loop<S: HighScoreStore>(mut shell: Shell<S>, fps: u32) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard::new(ratatui::restore);
    let frame = Duration::from_secs(1) / fps.max(1);
    let mut last = Instant::now();

    while !shell.app.quit {
        terminal.draw(|area| render(area, &shell.app))?;

        let action = if event::poll(frame)? {
            match event::read()? {
                Event::Key(key) => action_for_key(key),
                _ => None,
            }
        } else {
            None
        };

        // Measured, clamped dt: animation speed follows real time, but a long
        // stall (sleep, suspend) may not skip whole effects.
        let elapsed = last.elapsed().min(Duration::from_millis(MAX_DT_MILLIS));
        last = Instant::now();
        shell.apply(action, elapsed);
    }

    Ok(())
}

/// Enter the real terminal, draw a few frames, restore and exit.
///
/// This is the automated proof that raw mode and the alternate screen are set
/// up and restored under a pty. It never reads input and never persists.
pub fn run_self_test<S: HighScoreStore>(store: S, seed: u64) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard::new(ratatui::restore);
    let shell = Shell::start(store, seed);

    for _ in 0..5 {
        terminal.draw(|area| render(area, &shell.app))?;
        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Board, Direction};
    use crate::highscore::HighScoreError;
    use crate::render::frame_text;

    #[derive(Debug)]
    struct RecordingStore {
        value: u64,
        saves: Vec<u64>,
        fail: bool,
    }

    impl RecordingStore {
        fn new(value: u64) -> Self {
            Self {
                value,
                saves: Vec::new(),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                value: 0,
                saves: Vec::new(),
                fail: true,
            }
        }
    }

    impl HighScoreStore for RecordingStore {
        fn load(&self) -> u64 {
            self.value
        }

        fn save(&mut self, score: u64) -> Result<(), HighScoreError> {
            if self.fail {
                return Err(HighScoreError::NoDefaultPath);
            }
            self.saves.push(score);
            self.value = score;
            Ok(())
        }
    }

    fn pair_app() -> App {
        let board =
            Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        App::new(Game::from_state(board, 0, 0, 42))
    }

    #[test]
    fn shell_loads_best_from_its_store() {
        let shell = Shell::start(RecordingStore::new(5678), 42);

        assert_eq!(shell.app.game.best, 5678);
        assert_eq!(shell.store().saves, Vec::<u64>::new());
    }

    #[test]
    fn new_best_is_saved_once() {
        let mut shell = Shell::from_app(pair_app(), RecordingStore::new(0));

        shell.apply(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(shell.app.game.score, 2048);
        assert_eq!(shell.store().saves, vec![2048]);

        // Later frames with no new record must not write the store again.
        for _ in 0..10 {
            shell.apply(None, Duration::from_millis(33));
        }
        assert_eq!(shell.store().saves, vec![2048]);
    }

    #[test]
    fn clean_quit_saves_best() {
        let game = Game::from_state(Board::empty(), 1234, 0, 42);
        let mut shell = Shell::from_app(App::new(game), RecordingStore::new(0));

        shell.apply(Some(Action::Quit), Duration::from_millis(16));

        assert!(shell.app.quit);
        assert_eq!(shell.store().saves, vec![1234]);
    }

    #[test]
    fn failed_save_shows_footer_warning_for_three_seconds() {
        let mut shell = Shell::from_app(pair_app(), RecordingStore::failing());
        shell.apply(Some(Action::Move(Direction::Left)), Duration::ZERO);

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
}
