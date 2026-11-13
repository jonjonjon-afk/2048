//! Pure application state and the time-step update.
//!
//! This module never touches the filesystem, the clock or a terminal. Time
//! arrives as an injected `dt`; persistence needs are reported as effects and
//! executed by the shell. That is what makes the update deterministic.

use core::time::Duration;

use crate::action::Action;
use crate::effects::Effects;
use crate::game::Game;
use crate::highscore::{HighScores, NAME_MAX_LEN};
use crate::score::ScoreCount;

/// How long a failed save keeps its footer warning, in milliseconds.
pub const WARNING_MILLIS: u64 = 3_000;

/// A side effect the shell must perform after an update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Persist this named table (one accepted entry, M8).
    SaveScores(HighScores),
}

/// Everything the UI needs besides the game rules: quit latch, effects, table,
/// name entry and the footer warning.
#[derive(Clone, Debug)]
pub struct App {
    pub game: Game,
    pub quit: bool,
    /// Effect timeline, advanced only by the injected `dt`.
    pub effects: Effects,
    /// The named high-score table; the loaded best lives here.
    pub table: HighScores,
    /// True while the 26x15 table overlay should be drawn (M8).
    pub table_shown: bool,
    /// Displayed score that chases `game.score`; display state only.
    shown: ScoreCount,
    warning: Option<Duration>,
    /// The name being typed after a qualifying game over, if any.
    entry: Option<String>,
    /// True when the 20x12 score panel is visible, so H has nothing to toggle.
    panel_visible: bool,
}

impl App {
    /// Wrap a game in application state with an empty table.
    pub fn new(game: Game) -> Self {
        Self::with_table(game, HighScores::new())
    }

    /// Wrap a game and the loaded table; the in-game best is its top score.
    pub fn with_table(mut game: Game, table: HighScores) -> Self {
        game.best = game.best.max(table.best());
        Self {
            shown: ScoreCount::new(game.score),
            game,
            quit: false,
            effects: Effects::new(),
            table,
            table_shown: false,
            warning: None,
            entry: None,
            panel_visible: false,
        }
    }

    /// True while the `NEW HIGH SCORE` entry overlay is active.
    pub fn entry_active(&self) -> bool {
        self.entry.is_some()
    }

    /// The name typed so far, or `""` when entry is closed.
    pub fn entry_name(&self) -> &str {
        self.entry.as_deref().unwrap_or("")
    }

    /// Tell the state whether the wide-terminal panel is visible (shell only).
    pub fn set_panel_visible(&mut self, visible: bool) {
        self.panel_visible = visible;
    }

    /// Apply one action, start effects for the resulting merge/spawn/new best,
    /// then advance every timer by `dt`.
    ///
    /// Returns the effects the shell has to carry out (saving the named table).
    pub fn update(&mut self, action: Option<Action>, dt: Duration) -> Vec<Effect> {
        let mut effects = Vec::new();

        // While the entry overlay is active it owns the keyboard: no move,
        // restart, quit or toggle can happen through it (SPEC §5.5).
        if self.entry.is_some() {
            self.update_entry(action, &mut effects);
            self.advance(dt);
            return effects;
        }

        let best_before = self.game.best;
        let was_over = self.game.game_over;

        match action {
            Some(Action::Move(direction)) => {
                // Movement is ignored while the narrow-terminal table overlay
                // is open during play; H closes it, R and Q still work.
                if !self.overlay_blocks_moves() {
                    let outcome = self.game.apply_move(direction);
                    for merge in &outcome.merges {
                        self.effects.trigger_merge(*merge);
                    }
                    if let Some(spawn) = outcome.spawned {
                        self.effects.trigger_spawn(spawn.cell);
                    }
                }
            }
            Some(Action::Restart) => {
                self.game.restart();
                self.effects.clear();
                self.shown.reset(0);
                self.table_shown = false;
                self.entry = None;
            }
            Some(Action::Quit) => self.quit = true,
            Some(Action::DismissWin) => self.game.continue_after_win(),
            // Where the panel is visible, H does nothing (SPEC §5.5).
            Some(Action::ToggleTable) if !self.panel_visible => {
                self.table_shown = !self.table_shown;
            }
            _ => {}
        }

        // A new record pulses; nothing is written until a name is accepted.
        if self.game.best > best_before {
            self.effects.start_best_pulse();
        }

        // The game-over transition opens the table and, when the score
        // qualifies, the name entry (SPEC §5.3/§5.5, G17).
        if !was_over && self.game.game_over {
            self.table_shown = true;
            if self.table.qualifies(self.game.score) {
                self.entry = Some(String::new());
            }
        }

        // The action may have raised the real score; aim the shown value at it
        // before this update's `dt` advances the count.
        if self.game.score != self.shown.target() {
            self.shown.retarget(self.game.score);
        }

        self.advance(dt);
        effects
    }

    /// Handle one key action while the entry overlay is active.
    fn update_entry(&mut self, action: Option<Action>, effects: &mut Vec<Effect>) {
        match action {
            Some(Action::EntryChar(character)) => {
                if let Some(name) = self.entry.as_mut() {
                    if character.is_ascii_alphanumeric() && name.chars().count() < NAME_MAX_LEN {
                        name.push(character.to_ascii_uppercase());
                    }
                }
            }
            Some(Action::EntryBackspace) => {
                if let Some(name) = self.entry.as_mut() {
                    name.pop();
                }
            }
            Some(Action::EntryEnter) => {
                if self.entry.as_ref().is_some_and(|name| !name.is_empty()) {
                    if let Some(name) = self.entry.take() {
                        // The saved entry stores the real score, never the
                        // displayed count (SPEC §5.3).
                        self.table.insert(name, self.game.score);
                        effects.push(Effect::SaveScores(self.table.clone()));
                    }
                }
            }
            Some(Action::EntryCancel) => self.entry = None,
            _ => {}
        }
    }

    /// True while the table overlay is open on a terminal without the panel.
    fn overlay_blocks_moves(&self) -> bool {
        self.table_shown && !self.panel_visible
    }

    /// The score to display: the real score once the count has caught up.
    ///
    /// Every user-visible score text (panel edge, overlays) reads through this
    /// one accessor.
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

    fn table(rows: &[(&str, u64)]) -> HighScores {
        let mut table = HighScores::new();
        for (name, score) in rows {
            table.insert(*name, *score);
        }
        table
    }

    fn rows(table: &HighScores) -> Vec<(String, u64)> {
        table
            .entries()
            .iter()
            .map(|entry| (entry.name.clone(), entry.score))
            .collect()
    }

    fn full_table() -> HighScores {
        table(&[
            ("P0", 1000),
            ("P1", 900),
            ("P2", 800),
            ("P3", 700),
            ("P4", 600),
            ("P5", 500),
            ("P6", 400),
            ("P7", 300),
            ("P8", 200),
            ("P9", 50),
        ])
    }

    /// A full board with one merge available; the post-move spawn ends the game.
    fn merge_game_over_board() -> Board {
        Board::from_rows([
            [2, 2, 8, 16],
            [16, 32, 64, 128],
            [8, 16, 32, 64],
            [64, 128, 8, 32],
        ])
    }

    /// A full board with one gap; sliding fills it without scoring.
    fn no_score_game_over_board() -> Board {
        Board::from_rows([
            [0, 2, 4, 8],
            [16, 32, 64, 128],
            [8, 16, 32, 64],
            [64, 128, 8, 32],
        ])
    }

    fn transition_app(board: Board, score: u64, table: HighScores) -> App {
        App::with_table(Game::from_state(board, score, 0, 42), table)
    }

    fn entry_app() -> App {
        let mut app = transition_app(merge_game_over_board(), 0, HighScores::new());
        app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert!(app.game.game_over);
        assert_eq!(app.game.score, 4);
        assert!(app.entry_active());
        app
    }

    #[test]
    fn entry_opens_only_for_a_qualifying_score() {
        // A full table whose 10th score is 50: 0 and 50 do not qualify.
        let mut zero = transition_app(no_score_game_over_board(), 0, full_table());
        zero.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(zero.game.score, 0);
        assert!(zero.game.game_over);
        assert!(!zero.entry_active(), "score 0 must not open entry");

        let mut fifty = transition_app(merge_game_over_board(), 46, full_table());
        fifty.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(fifty.game.score, 50);
        assert!(fifty.game.game_over);
        assert!(!fifty.entry_active(), "50 is not greater than the 10th");

        let mut fifty_one = transition_app(merge_game_over_board(), 47, full_table());
        fifty_one.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(fifty_one.game.score, 51);
        assert!(fifty_one.game.game_over);
        assert!(fifty_one.entry_active(), "51 must beat the 10th");
        assert_eq!(fifty_one.entry_name(), "");

        // An empty table always qualifies a positive score.
        let empty = entry_app();
        assert_eq!(empty.game.score, 4);
        assert!(empty.entry_active());
    }

    #[test]
    fn entry_types_uppercase_and_caps_at_ten() {
        let mut app = entry_app();
        for character in ['a', 'b', '1', 'c', 'd'] {
            app.update(Some(Action::EntryChar(character)), Duration::ZERO);
        }
        assert_eq!(app.entry_name(), "AB1CD");

        for character in "efghijklmn".chars() {
            app.update(Some(Action::EntryChar(character)), Duration::ZERO);
        }
        assert_eq!(app.entry_name(), "AB1CDEFGHI");
        assert_eq!(app.entry_name().chars().count(), NAME_MAX_LEN);

        app.update(Some(Action::EntryBackspace), Duration::ZERO);
        assert_eq!(app.entry_name(), "AB1CDEFGH");
        app.update(Some(Action::EntryBackspace), Duration::ZERO);
        assert_eq!(app.entry_name(), "AB1CDEFG");
    }

    #[test]
    fn entry_enter_saves_the_real_score_and_emits_one_save() {
        let mut app = transition_app(merge_game_over_board(), 12, HighScores::new());
        app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);
        assert_eq!(app.game.score, 16);
        assert!(app.entry_active());
        let board = app.game.board;
        let rng = app.game.rng.state();
        let best = app.game.best;

        app.update(Some(Action::EntryChar('a')), Duration::ZERO);
        app.update(Some(Action::EntryChar('b')), Duration::ZERO);
        let effects = app.update(Some(Action::EntryEnter), Duration::ZERO);

        assert!(!app.entry_active(), "Enter must close the entry");
        assert_eq!(rows(&app.table), vec![("AB".to_owned(), 16)]);
        assert_eq!(effects.len(), 1, "exactly one table-save effect");
        assert_eq!(effects, vec![Effect::SaveScores(app.table.clone())]);
        assert!(!app.quit);
        assert_eq!(app.game.board, board, "saving must not touch the board");
        assert_eq!(app.game.rng.state(), rng);
        assert_eq!(app.game.best, best);
    }

    #[test]
    fn entry_enter_with_no_characters_does_nothing() {
        let mut app = entry_app();
        let effects = app.update(Some(Action::EntryEnter), Duration::ZERO);

        assert!(app.entry_active(), "empty Enter keeps the entry open");
        assert!(effects.is_empty(), "empty Enter emits no save");
        assert!(app.table.is_empty());
    }

    #[test]
    fn entry_esc_discards_without_saving() {
        let mut app = entry_app();
        for character in ['a', 'b', 'c'] {
            app.update(Some(Action::EntryChar(character)), Duration::ZERO);
        }
        let effects = app.update(Some(Action::EntryCancel), Duration::ZERO);

        assert!(!app.entry_active(), "Esc closes the entry");
        assert!(app.table.is_empty(), "Esc discards the typed name");
        assert!(effects.is_empty());
        assert!(!app.quit, "Esc must not quit from name entry");
    }

    #[test]
    fn entry_keys_never_restart_quit_or_toggle() {
        let mut app = entry_app();
        let board = app.game.board;
        let rng = app.game.rng.state();
        let shown = app.table_shown;

        for action in [
            Action::Restart,
            Action::Quit,
            Action::ToggleTable,
            Action::Move(Direction::Left),
            Action::Move(Direction::Right),
            Action::Move(Direction::Up),
            Action::Move(Direction::Down),
        ] {
            app.update(Some(action), Duration::ZERO);
            assert!(!app.quit, "{action:?} quit the game");
            assert_eq!(app.game.board, board, "{action:?} moved the board");
            assert_eq!(app.game.rng.state(), rng, "{action:?} used the RNG");
            assert_eq!(app.table_shown, shown, "{action:?} toggled the overlay");
            assert!(app.entry_active(), "{action:?} closed the entry");
        }

        // R, Q and H reach the field as characters, not as commands.
        for character in ['r', 'q', 'h'] {
            app.update(Some(Action::EntryChar(character)), Duration::ZERO);
        }
        assert_eq!(app.entry_name(), "RQH");
        assert!(!app.quit);
        assert_eq!(app.table_shown, shown);
    }

    #[test]
    fn a_new_best_starts_a_pulse_and_emits_no_save() {
        let board =
            Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut app = App::new(Game::from_state(board, 0, 0, 42));

        let effects = app.update(Some(Action::Move(Direction::Left)), Duration::ZERO);

        assert_eq!(app.game.score, 2048);
        assert_eq!(app.game.best, 2048);
        assert_eq!(app.effects.best_pulse_age(), Some(Duration::ZERO));
        assert!(effects.is_empty(), "a new best alone writes nothing");
    }

    #[test]
    fn quit_emits_no_save() {
        let mut app = App::new(Game::from_state(Board::empty(), 1234, 0, 42));
        let effects = app.update(Some(Action::Quit), Duration::ZERO);

        assert!(app.quit);
        assert!(effects.is_empty(), "an unnamed run is never persisted");
    }

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
        assert_eq!(first.table, second.table);
        assert_eq!(first.table_shown, second.table_shown);
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
