//! Headless command line: argument parsing and the `--dump` text format.
//!
//! M1 has no terminal setup. Dump mode always uses the in-memory high-score
//! store, so a seeded run can never read or write the real score file.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::{Direction, Game};
use crate::highscore::{HighScoreStore, MemoryStore};

/// Usage text for `--help` and for a bare invocation.
pub const USAGE: &str = "\
2048 — headless core

Usage: game2048 [options]

Options:
  --seed <u64>     deterministic RNG seed (default: system time)
  --moves <lrud>   apply moves in order (used with --dump)
  --dump           print the board state and exit
  --help, -h       show this help

The terminal UI arrives in M2. This build can dump the core state only.
";

/// A recoverable command-line error. The process exits with status 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    message: String,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// The human-readable reason.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

/// Parsed command-line options.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub seed: Option<u64>,
    pub moves: Vec<Direction>,
    pub dump: bool,
    pub help: bool,
}

/// Parse arguments (without the program name).
pub fn parse<I>(arguments: I) -> Result<Args, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = Args::default();
    let mut arguments = arguments.into_iter();

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--seed" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| CliError::new("--seed requires a u64 value"))?;
                parsed.seed = Some(value.parse::<u64>().map_err(|_| {
                    CliError::new(format!("invalid --seed value '{value}': expected a u64"))
                })?);
            }
            "--moves" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| CliError::new("--moves requires a sequence of l, r, u, d"))?;
                parsed.moves = value
                    .chars()
                    .map(|character| {
                        Direction::from_char(character).ok_or_else(|| {
                            CliError::new(format!(
                                "invalid move '{character}' in --moves: expected only l, r, u, d"
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
            }
            "--dump" => parsed.dump = true,
            "--help" | "-h" => parsed.help = true,
            other => {
                return Err(CliError::new(format!(
                    "unknown argument '{other}' (try --help)"
                )))
            }
        }
    }

    Ok(parsed)
}

/// A seed derived from the clock. Only the shell calls this; the core is pure.
pub fn default_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
}

/// Run a headless game and return the exact SPEC 5.6 dump text.
pub fn headless_dump(seed: u64, moves: &[Direction]) -> String {
    let store = MemoryStore::new(0);
    let mut game = Game::new(seed, store.load());
    for direction in moves {
        game.apply_move(*direction);
    }
    dump(&game)
}

/// Format a game as the exact SPEC 5.6 block.
pub fn dump(game: &Game) -> String {
    let mut text = String::from("board:\n");
    for row in game.board.rows() {
        text.push_str(&format!("{} {} {} {}\n", row[0], row[1], row[2], row[3]));
    }
    text.push_str(&format!("score: {}\n", game.score));
    text.push_str(&format!("best: {}\n", game.best));
    text.push_str(&format!("won: {}\n", game.won));
    text.push_str(&format!("game_over: {}\n", game.game_over));
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Result<Args, CliError> {
        parse(values.iter().map(|value| (*value).to_owned()))
    }

    #[test]
    fn parses_seed_moves_and_dump() {
        let parsed = args(&["--seed", "42", "--moves", "lrud", "--dump"]).unwrap();

        assert_eq!(parsed.seed, Some(42));
        assert_eq!(
            parsed.moves,
            vec![
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down
            ]
        );
        assert!(parsed.dump);
        assert!(!parsed.help);
    }

    #[test]
    fn help_flag_is_parsed() {
        assert!(args(&["--help"]).unwrap().help);
        assert!(args(&["-h"]).unwrap().help);
    }

    #[test]
    fn rejects_unknown_flag() {
        let error = args(&["--bogus"]).unwrap_err();
        assert!(error.message().contains("unknown argument"));
    }

    #[test]
    fn rejects_bad_moves_character() {
        let error = args(&["--moves", "lx"]).unwrap_err();
        assert!(error.message().contains("invalid move 'x'"));
    }

    #[test]
    fn rejects_bad_seed() {
        let error = args(&["--seed", "not-a-number"]).unwrap_err();
        assert!(error.message().contains("invalid --seed"));
    }

    #[test]
    fn rejects_missing_values() {
        assert!(args(&["--seed"]).is_err());
        assert!(args(&["--moves"]).is_err());
    }

    #[test]
    fn dump_has_the_exact_spec_shape() {
        let text = headless_dump(42, &[]);
        let expected = "\
board:
2 0 0 0
0 0 0 0
0 0 0 0
2 0 0 0
score: 0
best: 0
won: false
game_over: false
";
        assert_eq!(text, expected);
    }
}
