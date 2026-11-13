//! The pure 2048 rules.
//!
//! The board is a row-major `[u32; 16]`: cell `row * 4 + column`. A move
//! compresses each line, merges adjacent equal pairs exactly once, records the
//! merge destinations, and spawns one tile only when the board changed.

use crate::rng::Xorshift64;

/// Board side length.
pub const SIZE: usize = 4;
/// Number of cells on the board.
pub const CELLS: usize = SIZE * SIZE;
/// Tile value that triggers the win state.
pub const WIN_VALUE: u32 = 2_048;

/// Convert row and column to a cell index.
const fn index(row: usize, column: usize) -> usize {
    row * SIZE + column
}

/// The four move directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// Parse the single-character CLI form used by `--moves`.
    pub fn from_char(value: char) -> Option<Self> {
        match value {
            'l' => Some(Self::Left),
            'r' => Some(Self::Right),
            'u' => Some(Self::Up),
            'd' => Some(Self::Down),
            _ => None,
        }
    }
}

/// One merge, expressed as the destination cell and the merged value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Merge {
    pub cell: usize,
    pub value: u32,
}

/// One spawned tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub cell: usize,
    pub value: u32,
}

/// One tile's journey during a move: its source cell, destination cell and the
/// value it carried before the merge step (SPEC 5.4, M13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    pub from: usize,
    pub to: usize,
    pub value: u32,
}

/// What a single move did. `moved = false` means no spawn, no score, no RNG use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    pub moved: bool,
    pub score_gained: u64,
    pub merges: Vec<Merge>,
    pub spawned: Option<Spawn>,
    /// One track per occupied source cell, sorted by `from` ascending.
    pub tracks: Vec<Track>,
}

impl MoveOutcome {
    fn blocked() -> Self {
        Self {
            moved: false,
            score_gained: 0,
            merges: Vec::new(),
            spawned: None,
            tracks: Vec::new(),
        }
    }
}

/// The result of sliding one board, before the spawn step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideOutcome {
    pub board: Board,
    pub moved: bool,
    pub score_gained: u64,
    pub merges: Vec<Merge>,
    /// One track per occupied source cell, sorted by `from` ascending.
    pub tracks: Vec<Track>,
}

/// A 4x4 board of tile values; zero is an empty cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [u32; CELLS],
}

impl Board {
    /// An empty board.
    pub const fn empty() -> Self {
        Self { cells: [0; CELLS] }
    }

    /// Build a board from row-major cells.
    pub const fn from_cells(cells: [u32; CELLS]) -> Self {
        Self { cells }
    }

    /// Build a board from a nested array of rows.
    pub const fn from_rows(rows: [[u32; SIZE]; SIZE]) -> Self {
        Self {
            cells: [
                rows[0][0], rows[0][1], rows[0][2], rows[0][3], rows[1][0], rows[1][1], rows[1][2],
                rows[1][3], rows[2][0], rows[2][1], rows[2][2], rows[2][3], rows[3][0], rows[3][1],
                rows[3][2], rows[3][3],
            ],
        }
    }

    /// Read a cell by row-major index.
    pub fn cell(&self, cell: usize) -> u32 {
        self.cells[cell]
    }

    /// Write a cell by row-major index.
    pub fn set_cell(&mut self, cell: usize, value: u32) {
        self.cells[cell] = value;
    }

    /// The underlying row-major cells.
    pub fn cells(&self) -> &[u32; CELLS] {
        &self.cells
    }

    /// Rows for display and tests.
    pub fn rows(&self) -> [[u32; SIZE]; SIZE] {
        [
            [self.cells[0], self.cells[1], self.cells[2], self.cells[3]],
            [self.cells[4], self.cells[5], self.cells[6], self.cells[7]],
            [self.cells[8], self.cells[9], self.cells[10], self.cells[11]],
            [
                self.cells[12],
                self.cells[13],
                self.cells[14],
                self.cells[15],
            ],
        ]
    }

    /// Ascending list of empty cell indices.
    pub fn empty_cells(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(cell, value)| (*value == 0).then_some(cell))
            .collect()
    }

    /// True when no cell is empty.
    pub fn is_full(&self) -> bool {
        self.cells.iter().all(|value| *value != 0)
    }

    /// True when at least one cell holds `value`.
    pub fn has_value(&self, value: u32) -> bool {
        self.cells.contains(&value)
    }

    /// True when two orthogonally adjacent cells hold the same non-zero value.
    pub fn has_equal_neighbour(&self) -> bool {
        for row in 0..SIZE {
            for column in 0..SIZE {
                let value = self.cells[index(row, column)];
                if value == 0 {
                    continue;
                }
                if column + 1 < SIZE && self.cells[index(row, column + 1)] == value {
                    return true;
                }
                if row + 1 < SIZE && self.cells[index(row + 1, column)] == value {
                    return true;
                }
            }
        }
        false
    }

    /// Game over means the board is full and no neighbour can merge.
    pub fn is_game_over(&self) -> bool {
        self.is_full() && !self.has_equal_neighbour()
    }

    /// Slide and merge one board in a direction, without spawning.
    ///
    /// This is the pure stage honoured by the G1-G8 golden examples; the spawn
    /// step belongs to [`Game::apply_move`].
    pub fn slide(&self, direction: Direction) -> SlideOutcome {
        let mut board = Board::empty();
        let mut merges = Vec::new();
        let mut tracks = Vec::new();
        let mut score_gained = 0u64;

        for line in 0..SIZE {
            let positions = line_positions(line, direction);
            // Keep each source cell beside its value so the track records the
            // value the tile carried *before* the merge step.
            let sources: Vec<(usize, u32)> = positions
                .iter()
                .map(|cell| (*cell, self.cell(*cell)))
                .filter(|(_, value)| *value != 0)
                .collect();

            let mut merged_line = Vec::with_capacity(SIZE);
            let mut cursor = 0;
            while cursor < sources.len() {
                if cursor + 1 < sources.len() && sources[cursor].1 == sources[cursor + 1].1 {
                    let value = sources[cursor].1 * 2;
                    let cell = positions[merged_line.len()];
                    merges.push(Merge { cell, value });
                    score_gained += u64::from(value);
                    // Both sources share the destination of the merged tile.
                    tracks.push(Track {
                        from: sources[cursor].0,
                        to: cell,
                        value: sources[cursor].1,
                    });
                    tracks.push(Track {
                        from: sources[cursor + 1].0,
                        to: cell,
                        value: sources[cursor + 1].1,
                    });
                    merged_line.push(value);
                    cursor += 2;
                } else {
                    let cell = positions[merged_line.len()];
                    tracks.push(Track {
                        from: sources[cursor].0,
                        to: cell,
                        value: sources[cursor].1,
                    });
                    merged_line.push(sources[cursor].1);
                    cursor += 1;
                }
            }

            for (slot, value) in merged_line.into_iter().enumerate() {
                board.set_cell(positions[slot], value);
            }
        }

        // Rows are visited top-to-bottom; a right/down move emits destinations
        // in reverse order, so sort the tracks by source for every direction.
        tracks.sort_by_key(|track| track.from);

        SlideOutcome {
            board,
            moved: board != *self,
            score_gained,
            merges,
            tracks,
        }
    }
}

/// A complete game: board, score, best, win/game-over state and RNG.
#[derive(Clone, Debug)]
pub struct Game {
    pub board: Board,
    pub score: u64,
    pub best: u64,
    /// True while the win overlay is visible.
    pub won: bool,
    /// Latched once 2048 is reached; continue never re-opens the overlay.
    pub reached_2048: bool,
    pub game_over: bool,
    pub rng: Xorshift64,
    /// The most recent spawn, for the TUI's spawn-pop effect later.
    pub last_spawn: Option<Spawn>,
}

impl Game {
    /// Start a new game with two spawned tiles and the supplied best score.
    pub fn new(seed: u64, best: u64) -> Self {
        let mut game = Self {
            board: Board::empty(),
            score: 0,
            best,
            won: false,
            reached_2048: false,
            game_over: false,
            rng: Xorshift64::new(seed),
            last_spawn: None,
        };
        game.spawn();
        game.spawn();
        game.game_over = game.board.is_game_over();
        game
    }

    /// Build a game from an explicit board without spawning. Used by tests.
    pub fn from_state(board: Board, score: u64, best: u64, seed: u64) -> Self {
        let reached_2048 = board.has_value(WIN_VALUE);
        Self {
            board,
            score,
            best: best.max(score),
            won: false,
            reached_2048,
            game_over: board.is_game_over(),
            rng: Xorshift64::new(seed),
            last_spawn: None,
        }
    }

    /// Reset the board and score but keep the best score (the R key contract).
    pub fn restart(&mut self) {
        self.board = Board::empty();
        self.score = 0;
        self.won = false;
        self.reached_2048 = false;
        self.game_over = false;
        self.last_spawn = None;
        self.spawn();
        self.spawn();
        self.game_over = self.board.is_game_over();
    }

    /// Dismiss the win overlay and keep playing.
    pub fn continue_after_win(&mut self) {
        self.won = false;
    }

    /// Spawn one tile using the documented two-draw order.
    pub fn spawn(&mut self) -> Option<Spawn> {
        let empties = self.board.empty_cells();
        if empties.is_empty() {
            self.last_spawn = None;
            return None;
        }
        let index = (self.rng.next_u64() % empties.len() as u64) as usize;
        let cell = empties[index];
        let value = if self.rng.next_u64().is_multiple_of(10) {
            4
        } else {
            2
        };
        self.board.set_cell(cell, value);
        let spawned = Spawn { cell, value };
        self.last_spawn = Some(spawned);
        Some(spawned)
    }

    /// Apply one move. Move keys are ignored while the win overlay is visible
    /// or the game is over, matching the key contract in SPEC 5.5.
    pub fn apply_move(&mut self, direction: Direction) -> MoveOutcome {
        if self.won || self.game_over {
            return MoveOutcome::blocked();
        }

        let slid = self.board.slide(direction);
        if !slid.moved {
            self.game_over = self.board.is_game_over();
            return MoveOutcome::blocked();
        }

        self.board = slid.board;
        self.score += slid.score_gained;
        self.best = self.best.max(self.score);

        if !self.reached_2048 && self.board.has_value(WIN_VALUE) {
            self.reached_2048 = true;
            self.won = true;
        }

        let spawned = self.spawn();
        self.game_over = self.board.is_game_over();

        MoveOutcome {
            moved: true,
            score_gained: slid.score_gained,
            merges: slid.merges,
            spawned,
            tracks: slid.tracks,
        }
    }
}

/// Cell indices of one row or column in the order tiles move.
fn line_positions(line: usize, direction: Direction) -> [usize; SIZE] {
    match direction {
        Direction::Left => [
            index(line, 0),
            index(line, 1),
            index(line, 2),
            index(line, 3),
        ],
        Direction::Right => [
            index(line, 3),
            index(line, 2),
            index(line, 1),
            index(line, 0),
        ],
        Direction::Up => [
            index(0, line),
            index(1, line),
            index(2, line),
            index(3, line),
        ],
        Direction::Down => [
            index(3, line),
            index(2, line),
            index(1, line),
            index(0, line),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(seed: u64) -> Game {
        Game::new(seed, 0)
    }

    #[test]
    fn left_merges_a_pair_once() {
        // G1: row 2 2 0 0, move left -> 4 0 0 0, score +4, one merge at cell 0.
        let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert!(outcome.moved);
        assert_eq!(outcome.board.rows()[0], [4, 0, 0, 0]);
        assert_eq!(outcome.score_gained, 4);
        assert_eq!(outcome.merges, vec![Merge { cell: 0, value: 4 }]);
        assert_eq!(outcome.merges.len(), 1);
    }

    #[test]
    fn double_pair_merges_both_pairs() {
        // G2: 2 2 2 2 -> 4 4 0 0, score +8, two merges.
        let board = Board::from_rows([[2, 2, 2, 2], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(outcome.board.rows()[0], [4, 4, 0, 0]);
        assert_eq!(outcome.score_gained, 8);
        assert_eq!(outcome.merges.len(), 2);
        assert_eq!(
            outcome.merges,
            vec![Merge { cell: 0, value: 4 }, Merge { cell: 1, value: 4 }]
        );

        // G5: 2 2 4 4 -> 4 8 0 0, score +12, two merges.
        let board = Board::from_rows([[2, 2, 4, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(outcome.board.rows()[0], [4, 8, 0, 0]);
        assert_eq!(outcome.score_gained, 12);
        assert_eq!(outcome.merges.len(), 2);
        assert_eq!(
            outcome.merges,
            vec![Merge { cell: 0, value: 4 }, Merge { cell: 1, value: 8 }]
        );
    }

    #[test]
    fn merged_tile_does_not_merge_again_in_the_same_move() {
        // G3: 4 4 8 0 -> 8 8 0 0; the new 8 must not chain with the old 8.
        let board = Board::from_rows([[4, 4, 8, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(outcome.board.rows()[0], [8, 8, 0, 0]);
        assert_eq!(outcome.score_gained, 8);
        assert_eq!(outcome.merges, vec![Merge { cell: 0, value: 8 }]);
    }

    #[test]
    fn gaps_are_closed_before_merging() {
        // G4: 2 0 2 4 -> 4 4 0 0; the gap closes before the merge.
        let board = Board::from_rows([[2, 0, 2, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(outcome.board.rows()[0], [4, 4, 0, 0]);
        assert_eq!(outcome.score_gained, 4);
        assert_eq!(outcome.merges, vec![Merge { cell: 0, value: 4 }]);
    }

    #[test]
    fn right_moves_toward_the_last_cell() {
        // G7: 2 2 0 0 -> 0 0 0 4, merge at cell 3.
        let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Right);

        assert_eq!(outcome.board.rows()[0], [0, 0, 0, 4]);
        assert_eq!(outcome.merges, vec![Merge { cell: 3, value: 4 }]);
    }

    #[test]
    fn vertical_moves_work_per_column() {
        // G8: column 0 has 2 2 0 0 (cells 0 and 4); move up -> cell 0 = 4.
        let board = Board::from_rows([[2, 0, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Up);

        assert_eq!(outcome.board.cell(0), 4);
        assert_eq!(outcome.board.cell(4), 0);
        assert_eq!(outcome.merges, vec![Merge { cell: 0, value: 4 }]);

        // Down: 2 2 0 0 becomes 0 0 0 4 in the column.
        let board = Board::from_rows([[2, 0, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Down);

        assert_eq!(outcome.board.cell(12), 4);
        assert_eq!(outcome.board.cell(8), 0);
        assert_eq!(outcome.merges, vec![Merge { cell: 12, value: 4 }]);
    }

    #[test]
    fn blocked_move_changes_nothing_and_does_not_spawn() {
        // G6: 2 4 2 4 already left-aligned; nothing moves, nothing spawns.
        let board = Board::from_rows([[2, 4, 2, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 0, 0, 42);
        let before = game.board;
        let before_rng = game.rng.state();
        let outcome = game.apply_move(Direction::Left);

        assert!(!outcome.moved);
        assert!(outcome.merges.is_empty());
        assert_eq!(outcome.score_gained, 0);
        assert!(outcome.spawned.is_none());
        assert_eq!(game.board, before);
        assert_eq!(game.score, 0);
        assert_eq!(game.rng.state(), before_rng);
    }

    #[test]
    fn game_over_when_board_is_full_and_no_neighbour_matches() {
        // G11: a full checkerboard has no equal orthogonal neighbours.
        let board = Board::from_rows([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        let mut game = Game::from_state(board, 0, 0, 42);

        assert!(game.game_over);
        let before = game.board;
        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            let outcome = game.apply_move(direction);
            assert!(!outcome.moved);
            assert!(outcome.spawned.is_none());
            assert_eq!(game.board, before);
            assert!(game.game_over);
        }
    }

    #[test]
    fn win_is_detected_and_continue_dismisses_the_overlay() {
        // G10: 1024 1024 -> 2048; won; continue hides the overlay exactly once.
        let board =
            Board::from_rows([[1024, 1024, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 0, 0, 42);
        let outcome = game.apply_move(Direction::Left);

        assert_eq!(game.board.cell(0), 2048);
        assert_eq!(
            outcome.merges,
            vec![Merge {
                cell: 0,
                value: 2048
            }]
        );
        assert!(game.reached_2048);
        assert!(game.won);

        game.continue_after_win();
        assert!(!game.won);
        assert!(game.reached_2048);
        assert!(!game.game_over);
    }

    #[test]
    fn spawn_uses_the_documented_rng_order() {
        // G9: seed 42, first spawn at cell 0 = 2, second at cell 12 = 2.
        let game = game(42);

        assert_eq!(game.board.cell(0), 2);
        assert_eq!(game.board.cell(12), 2);
        assert_eq!(game.board.empty_cells().len(), 14);
        assert_eq!(game.last_spawn, Some(Spawn { cell: 12, value: 2 }));
    }

    #[test]
    fn blocked_move_does_not_advance_rng() {
        let board = Board::from_rows([[2, 4, 2, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 0, 0, 7);
        let before = game.rng.state();

        game.apply_move(Direction::Left);

        assert_eq!(game.rng.state(), before);
    }

    #[test]
    fn new_game_starts_with_exactly_two_tiles_and_keeps_best() {
        let game = Game::new(42, 1234);
        let occupied = game
            .board
            .cells()
            .iter()
            .filter(|value| **value != 0)
            .count();

        assert_eq!(occupied, 2);
        assert_eq!(game.best, 1234);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut first = Xorshift64::new(99);
        let mut second = Xorshift64::new(99);
        for _ in 0..32 {
            assert_eq!(first.next_u64(), second.next_u64());
        }

        let mut first_game = game(123);
        let mut second_game = game(123);
        for direction in [
            Direction::Left,
            Direction::Down,
            Direction::Right,
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
            Direction::Up,
        ] {
            let first_outcome = first_game.apply_move(direction);
            let second_outcome = second_game.apply_move(direction);
            assert_eq!(first_outcome, second_outcome);
            assert_eq!(first_game.board, second_game.board);
            assert_eq!(first_game.score, second_game.score);
        }
    }

    #[test]
    fn changed_move_spawns_exactly_one_tile() {
        let board = Board::from_rows([[2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let mut game = Game::from_state(board, 0, 0, 42);
        let occupied_before = game.board.cells().iter().filter(|v| **v != 0).count();

        let outcome = game.apply_move(Direction::Right);
        let occupied_after = game.board.cells().iter().filter(|v| **v != 0).count();

        assert!(outcome.moved);
        assert!(outcome.spawned.is_some());
        assert_eq!(occupied_after, occupied_before + 1);
    }

    fn track(from: usize, to: usize, value: u32) -> Track {
        Track { from, to, value }
    }

    #[test]
    fn slide_tracks_a_single_merge() {
        // G20: row 2 2 0 0 left -> both sources keep their own value and share
        // the merge destination cell 0.
        let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(
            outcome.tracks,
            vec![track(0, 0, 2), track(1, 0, 2)],
            "G20 slide tracks"
        );
        assert_eq!(outcome.board.rows()[0], [4, 0, 0, 0]);
        assert_eq!(outcome.merges, vec![Merge { cell: 0, value: 4 }]);
        assert!(outcome.moved);
    }

    #[test]
    fn slide_tracks_keep_gaps_and_destination_order() {
        // G4: 2 0 2 4 left -> 4 4 0 0; the gap closes before the merge and the
        // 4 lands in the second slot.
        let board = Board::from_rows([[2, 0, 2, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        // Both 2s merge into cell 0, so they share that destination; the old 4
        // ends in cell 1 (the review's G4 row listed 1 and 2, which contradicts
        // the G4 board `4 4 0 0` and the shared-destination rule of G20).
        assert_eq!(
            outcome.tracks,
            vec![track(0, 0, 2), track(2, 0, 2), track(3, 1, 4)]
        );
        assert_eq!(outcome.board.rows()[0], [4, 4, 0, 0]);
        assert_eq!(outcome.score_gained, 4);
    }

    #[test]
    fn slide_tracks_a_right_move() {
        // G7: 2 2 0 0 right -> 0 0 0 4; both sources travel to cell 3 and the
        // vector is sorted by the source cell, not the travel order.
        let board = Board::from_rows([[2, 2, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Right);

        assert_eq!(
            outcome.tracks,
            vec![track(0, 3, 2), track(1, 3, 2)],
            "tracks ascending by from"
        );
        assert_eq!(outcome.board.rows()[0], [0, 0, 0, 4]);
        assert_eq!(outcome.merges, vec![Merge { cell: 3, value: 4 }]);
    }

    #[test]
    fn slide_tracks_a_vertical_merge() {
        // G8: column 2,2,0,0 up -> cell 0 = 4; the two sources are cells 0 and 4.
        let board = Board::from_rows([[2, 0, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Up);

        assert_eq!(outcome.tracks, vec![track(0, 0, 2), track(4, 0, 2)]);
        assert_eq!(outcome.board.cell(0), 4);
        assert_eq!(outcome.board.cell(4), 0);
    }

    #[test]
    fn slide_tracks_an_unchanged_row() {
        // G6: 2 4 2 4 already left-aligned; every occupied cell still gets a
        // track, but nothing moved and no merge happened.
        let board = Board::from_rows([[2, 4, 2, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let outcome = board.slide(Direction::Left);

        assert_eq!(
            outcome.tracks,
            vec![
                track(0, 0, 2),
                track(1, 1, 4),
                track(2, 2, 2),
                track(3, 3, 4)
            ]
        );
        assert!(!outcome.moved);
        assert_eq!(outcome.score_gained, 0);
    }

    #[test]
    fn every_occupied_cell_gets_exactly_one_track() {
        for seed in 1..=12u64 {
            let game = Game::new(seed, 0);
            let occupied: Vec<usize> = game
                .board
                .cells()
                .iter()
                .enumerate()
                .filter_map(|(cell, value)| (*value != 0).then_some(cell))
                .collect();
            let tracks = game.board.slide(Direction::Left).tracks;

            assert_eq!(tracks.len(), occupied.len(), "seed {seed} track count");
            let sources: Vec<usize> = tracks.iter().map(|track| track.from).collect();
            assert_eq!(
                sources, occupied,
                "seed {seed} sources are the occupied cells"
            );
            for pair in sources.windows(2) {
                assert!(pair[0] < pair[1], "seed {seed} sources are ascending");
            }
        }
    }

    #[test]
    fn restart_resets_the_board_and_keeps_best() {
        let mut game = Game::new(42, 500);
        game.score = 300;
        game.best = 500;

        game.restart();

        assert_eq!(game.score, 0);
        assert_eq!(game.best, 500);
        assert!(!game.won);
        assert!(!game.reached_2048);
        assert_eq!(
            game.board
                .cells()
                .iter()
                .filter(|value| **value != 0)
                .count(),
            2
        );
    }
}
