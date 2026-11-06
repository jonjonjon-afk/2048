//! Key-event to action mapping.
//!
//! This is the only module that reads crossterm event types. Release events
//! never produce an action; only presses and auto-repeats do.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::action::Action;
use crate::game::Direction;

/// Translate one key event into an action, or `None` when the key is unbound.
///
/// Key releases are ignored so that terminals which report both edges of a
/// keypress do not apply a move twice.
pub fn action_for_key(key: KeyEvent) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }

    match key.code {
        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
            Some(Action::Move(Direction::Left))
        }
        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
            Some(Action::Move(Direction::Right))
        }
        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::Move(Direction::Up)),
        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
            Some(Action::Move(Direction::Down))
        }
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Action::Restart),
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char(' ') | KeyCode::Enter => Some(Action::DismissWin),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

    use super::*;
    use crate::game::{Board, Game};
    use crate::update::App;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app_with_win() -> App {
        let board = Board::from_rows([[2048, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 0, 500, 42);
        game.won = true;
        game.reached_2048 = true;
        App::new(game)
    }

    #[test]
    fn arrows_and_wasd_map_to_the_four_directions() {
        let expected = [
            (KeyCode::Left, Direction::Left),
            (KeyCode::Right, Direction::Right),
            (KeyCode::Up, Direction::Up),
            (KeyCode::Down, Direction::Down),
            (KeyCode::Char('a'), Direction::Left),
            (KeyCode::Char('d'), Direction::Right),
            (KeyCode::Char('w'), Direction::Up),
            (KeyCode::Char('s'), Direction::Down),
        ];

        for (code, direction) in expected {
            assert_eq!(
                action_for_key(press(code)),
                Some(Action::Move(direction)),
                "key {code:?} must map to {direction:?}"
            );
        }
    }

    #[test]
    fn wasd_accepts_upper_and_lower_case() {
        assert_eq!(
            action_for_key(press(KeyCode::Char('A'))),
            Some(Action::Move(Direction::Left))
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('D'))),
            Some(Action::Move(Direction::Right))
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('W'))),
            Some(Action::Move(Direction::Up))
        );
        assert_eq!(
            action_for_key(press(KeyCode::Char('S'))),
            Some(Action::Move(Direction::Down))
        );
    }

    #[test]
    fn r_restarts_and_q_or_esc_quits() {
        let mut game = Game::new(42, 500);
        game.score = 300;
        game.best = 500;
        let mut app = App::new(game);
        let action = action_for_key(press(KeyCode::Char('r')));
        assert_eq!(action, Some(Action::Restart));

        app.update(action, std::time::Duration::ZERO);
        assert_eq!(app.game.score, 0, "R must reset the score");
        assert_eq!(app.game.best, 500, "R must keep the best score");
        assert!(!app.quit);

        let mut quit_app = App::new(Game::new(42, 0));
        assert_eq!(
            action_for_key(press(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        quit_app.update(
            action_for_key(press(KeyCode::Char('q'))),
            std::time::Duration::ZERO,
        );
        assert!(quit_app.quit, "Q must set quit");

        let mut escape_app = App::new(Game::new(42, 0));
        assert_eq!(action_for_key(press(KeyCode::Esc)), Some(Action::Quit));
        escape_app.update(
            action_for_key(press(KeyCode::Esc)),
            std::time::Duration::ZERO,
        );
        assert!(escape_app.quit, "Esc must set quit");
    }

    #[test]
    fn space_or_enter_dismisses_the_win_overlay() {
        for code in [KeyCode::Char(' '), KeyCode::Enter] {
            let mut app = app_with_win();
            assert!(app.game.won);
            let action = action_for_key(press(code));
            assert_eq!(action, Some(Action::DismissWin), "key {code:?}");

            app.update(action, std::time::Duration::ZERO);
            assert!(!app.game.won, "key {code:?} must dismiss the overlay");
            assert!(!app.game.game_over);
        }

        // A dismiss action outside the win state changes no game state.
        let mut app = App::new(Game::new(42, 0));
        let before = app.game.board;
        let before_rng = app.game.rng.state();
        let action = action_for_key(press(KeyCode::Enter));
        app.update(action, std::time::Duration::ZERO);
        assert_eq!(app.game.board, before);
        assert_eq!(app.game.score, 0);
        assert_eq!(app.game.rng.state(), before_rng);
    }

    #[test]
    fn r_and_q_still_work_while_the_win_overlay_is_visible() {
        let mut restart_app = app_with_win();
        restart_app.game.score = 2048;
        restart_app.game.best = 9000;
        let action = action_for_key(press(KeyCode::Char('r')));
        assert_eq!(action, Some(Action::Restart));

        restart_app.update(action, std::time::Duration::ZERO);
        assert_eq!(restart_app.game.score, 0, "R must reset the score");
        assert_eq!(restart_app.game.best, 9000, "R must keep the best");
        assert!(!restart_app.game.won, "R must close the win overlay");
        assert_eq!(
            restart_app
                .game
                .board
                .cells()
                .iter()
                .filter(|value| **value != 0)
                .count(),
            2
        );

        for code in [KeyCode::Char('q'), KeyCode::Esc] {
            let mut quit_app = app_with_win();
            let action = action_for_key(press(code));
            assert_eq!(action, Some(Action::Quit), "key {code:?}");
            quit_app.update(action, std::time::Duration::ZERO);
            assert!(quit_app.quit, "key {code:?} must quit from the overlay");
        }
    }

    #[test]
    fn key_release_events_do_not_trigger_actions() {
        for code in [
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('a'),
            KeyCode::Char('R'),
            KeyCode::Char('q'),
            KeyCode::Esc,
            KeyCode::Char(' '),
            KeyCode::Enter,
        ] {
            let mut key = KeyEvent::new(code, KeyModifiers::NONE);
            key.kind = KeyEventKind::Release;
            assert_eq!(action_for_key(key), None, "release of {code:?}");
        }
    }

    #[test]
    fn unbound_keys_do_nothing() {
        assert_eq!(action_for_key(press(KeyCode::Char('z'))), None);
        assert_eq!(action_for_key(press(KeyCode::Tab)), None);
        assert_eq!(action_for_key(press(KeyCode::Backspace)), None);
    }
}
