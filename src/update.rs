//! Pure application state and the time-step update.
//!
//! This module never touches the filesystem, the clock or a terminal. Time
//! arrives as an injected `dt`; persistence needs are reported as effects and
//! executed by the shell. That is what makes the update deterministic.

use core::time::Duration;

use crate::action::Action;
use crate::effects::Effects;
use crate::game::Game;
use crate::score::ScoreCount;

/// How long a failed save keeps its footer warning, in milliseconds.
pub const WARNING_MILLIS: u64 = 3_000;

/// A side effect the shell must perform after an update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Persist this best score.
    SaveBest(u64),
}

/// Everything the UI needs besides the game rules: quit latch, effects and warning.
#[derive(Clone, Debug)]
pub struct App {
    pub game: Game,
    pub quit: bool,
    /// Effect timeline, advanced only by the injected `dt`.
    pub effects: Effects,
    /// Displayed score that chases `game.score`; display state only.
    shown: ScoreCount,
    warning: Option<Duration>,
}

impl App {
    /// Wrap a game in application state.
    pub fn new(game: Game) -> Self {
        Self {
            shown: ScoreCount::new(game.score),
            game,
            quit: false,
            effects: Effects::new(),
            warning: None,
        }
    }

    /// Apply one action, start effects for the resulting merge/spawn/new best,
    /// then advance every timer by `dt`.
    ///
    /// Returns the effects the shell has to carry out (saving the best score).
    pub fn update(&mut self, action: Option<Action>, dt: Duration) -> Vec<Effect> {
        let mut effects = Vec::new();
        let best_before = self.game.best;

        match action {
            Some(Action::Move(direction)) => {
                let outcome = self.game.apply_move(direction);
                for merge in &outcome.merges {
                    self.effects.trigger_merge(*merge);
                }
                if let Some(spawn) = outcome.spawned {
                    self.effects.trigger_spawn(spawn.cell);
                }
            }
            Some(Action::Restart) => {
                self.game.restart();
                self.effects.clear();
                self.shown.reset(0);
            }
            Some(Action::Quit) => {
                self.quit = true;
                effects.push(Effect::SaveBest(self.game.best));
            }
            Some(Action::DismissWin) => self.game.continue_after_win(),
            None => {}
        }

        // A new record is written immediately, once per new value.
        if self.game.best > best_before {
            self.effects.start_best_pulse();
            effects.push(Effect::SaveBest(self.game.best));
        }

        // The action may have raised the real score; aim the shown value at it
        // before this update's `dt` advances the count.
        if self.game.score != self.shown.target() {
            self.shown.retarget(self.game.score);
        }

        self.advance(dt);
        effects
    }

    /// The score to display: the real score once the count has caught up.
    ///
    /// Every user-visible score text (panel edge, game-over overlay) reads
    /// through this one accessor.
    pub fn shown_score(&self) -> u64 {
        self.shown.shown()
    }

    /// Record that the store rejected a save and start the footer warning.
    pub fn report_save_failure(&mut self) {
        self.warning = Some(Duration::from_millis(WARNING_MILLIS));
    }

    /// True while the footer must show the storage warning.
    pub fn warning_active(&self) -> bool {
        self.warning.is_some_and(|remaining| !remaining.is_zero())
    }

    /// Advance the effect clock, the score count and the warning timer by `dt`.
    fn advance(&mut self, dt: Duration) {
        self.effects.advance(dt);
        self.shown.advance(dt);

        let Some(remaining) = self.warning else {
            return;
        };
        let next = remaining.saturating_sub(dt);
        self.warning = if next.is_zero() || next <= dt {
            None
        } else {
            Some(next)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Board, Direction};

    fn game_over_app() -> App {
        let board = Board::from_rows([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        App::new(Game::from_state(board, 100, 100, 42))
    }

    fn win_overlay_app() -> App {
        let board = Board::from_rows([[2048, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 2048, 2048, 42);
        game.won = true;
        game.reached_2048 = true;
        App::new(game)
    }

    #[test]
    fn move_keys_are_ignored_while_the_win_overlay_is_visible() {
        let mut app = win_overlay_app();
        let board = app.game.board;
        let rng = app.game.rng.state();
        let score = app.game.score;

        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            app.update(Some(Action::Move(direction)), Duration::ZERO);
            assert_eq!(
                app.game.board, board,
                "move {direction:?} changed the board"
            );
            assert_eq!(app.game.score, score);
            assert_eq!(app.game.rng.state(), rng);
            assert!(app.game.won);
        }
    }

    #[test]
    fn move_keys_are_ignored_after_game_over() {
        let mut app = game_over_app();
        let board = app.game.board;
        let rng = app.game.rng.state();
        let score = app.game.score;

        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            app.update(Some(Action::Move(direction)), Duration::ZERO);
            assert_eq!(app.game.board, board);
            assert_eq!(app.game.score, score);
            assert_eq!(app.game.rng.state(), rng);
            assert!(app.game.game_over);
        }
    }

    #[test]
    fn same_actions_and_dt_produce_identical_state() {
        let mut first = App::new(Game::new(42, 0));
        let mut second = App::new(Game::new(42, 0));
        let actions = [
            Some(Action::Move(Direction::Left)),
            Some(Action::Move(Direction::Down)),
            Some(Action::Move(Direction::Right)),
            Some(Action::DismissWin),
            Some(Action::Move(Direction::Up)),
            None,
            Some(Action::Restart),
            Some(Action::Move(Direction::Left)),
        ];
        let dt = Duration::from_millis(16);

        for action in actions {
            let first_effects = first.update(action, dt);
            let second_effects = second.update(action, dt);
            assert_eq!(first_effects, second_effects);
            assert_eq!(first.game.board, second.game.board);
            assert_eq!(first.game.score, second.game.score);
            assert_eq!(first.game.best, second.game.best);
            assert_eq!(first.game.won, second.game.won);
            assert_eq!(first.game.game_over, second.game.game_over);
            assert_eq!(first.game.rng.state(), second.game.rng.state());
        }
        assert_eq!(first.quit, second.quit);
        assert_eq!(first.warning_active(), second.warning_active());
    }

    #[test]
    fn a_new_best_reports_exactly_one_save() {
        let board =
            Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut app = App::new(Game::from_state(board, 0, 0, 42));

        let effects = app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(effects, vec![Effect::SaveBest(2048)]);
        assert_eq!(app.game.score, 2048);
        assert_eq!(app.game.best, 2048);
    }

    #[test]
    fn the_warning_expires_after_three_seconds() {
        let mut app = App::new(Game::new(42, 0));
        app.report_save_failure();
        assert!(app.warning_active());

        let frame = Duration::from_millis(1_000 / 30);
        for _ in 0..90 {
            app.update(None, frame);
        }
        assert!(!app.warning_active(), "90 frames at 30 fps is 3000 ms");
    }
}
