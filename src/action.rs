//! The small set of player actions shared by the input mapper and the update.
//!
//! Nothing here knows about terminals, keys or storage: the input layer turns
//! key events into actions, and the update layer applies them to the state.

use crate::game::Direction;

/// One player action, produced by the key mapper and consumed by `App::update`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Slide and merge the board in one direction.
    Move(Direction),
    /// Start a new game, keeping the best score.
    Restart,
    /// Leave the game cleanly.
    Quit,
    /// Dismiss the win overlay and keep playing.
    DismissWin,
    /// Toggle the high-score table overlay (SPEC §5.5, M8).
    ToggleTable,
    /// Append one uppercase character to the name-entry field.
    EntryChar(char),
    /// Delete the last character of the name-entry field.
    EntryBackspace,
    /// Save the typed name and close the entry overlay.
    EntryEnter,
    /// Discard the typed name and close the entry overlay.
    EntryCancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_compare_by_value() {
        assert_eq!(Action::Move(Direction::Left), Action::Move(Direction::Left));
        assert_ne!(Action::Move(Direction::Left), Action::Move(Direction::Up));
        assert_ne!(Action::Restart, Action::Quit);
        assert_eq!(Action::EntryChar('A'), Action::EntryChar('A'));
        assert_ne!(Action::EntryChar('A'), Action::EntryChar('B'));
    }
}
