//! Command line: argument parsing, seed resolution and the `--dump` text.
//!
//! Dump, render-once and self-test modes never open the real high-score file.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game::{Direction, Game};
use crate::highscore::{HighScoreStore, MemoryStore};

/// Live frame rate used when `--fps` is absent.
pub const DEFAULT_FPS: u32 = 60;
/// Seed used by the demo boards when `--seed` is absent.
pub const DEMO_SEED: u64 = 42;

/// Usage text for `--help` and for a bare invocation.
pub const USAGE: &str = "\
2048 — a keyboard-first terminal game

Usage: game2048 [options]

Options:
  --seed <u64>     deterministic RNG seed (default: system time; demo: 42)
  --moves <lrud>   apply moves in order (used with --dump)
  --dump           print the board state and exit
  --render-once    print one 100x36 plain-text frame and exit
  --frame <n>      animation frame for --render-once (16 ms steps)
  --demo           seeded showcase board with an in-memory score
  --demo-merge     demo board with a merge ready for captures
  --no-persist     force the in-memory high-score store
  --self-test-tui  enter the TUI, draw 120 paced frames, restore and exit
  --fps <n>        live frame rate (default 60)
  --help, -h       show this help
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Args {
    pub seed: Option<u64>,
    pub moves: Vec<Direction>,
    pub dump: bool,
    pub help: bool,
    pub render_once: bool,
    pub self_test_tui: bool,
    pub no_persist: bool,
    pub demo: bool,
    pub demo_merge: bool,
    pub fps: u32,
    pub frame: Option<u64>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            seed: None,
            moves: Vec::new(),
            dump: false,
            help: false,
            render_once: false,
            self_test_tui: false,
            no_persist: false,
            demo: false,
            demo_merge: false,
            fps: DEFAULT_FPS,
            frame: None,
        }
    }
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
            "--render-once" => parsed.render_once = true,
            "--self-test-tui" => parsed.self_test_tui = true,
            "--no-persist" => parsed.no_persist = true,
            "--demo" => parsed.demo = true,
            "--demo-merge" => parsed.demo_merge = true,
            "--fps" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| CliError::new("--fps requires a positive number"))?;
                let fps = value.parse::<u32>().map_err(|_| {
                    CliError::new(format!(
                        "invalid --fps value '{value}': expected a positive u32"
                    ))
                })?;
                if fps == 0 {
                    return Err(CliError::new(
                        "invalid --fps value '0': expected a positive frame rate",
                    ));
                }
                parsed.fps = fps;
            }
            "--frame" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| CliError::new("--frame requires a u64 value"))?;
                parsed.frame = Some(value.parse::<u64>().map_err(|_| {
                    CliError::new(format!("invalid --frame value '{value}': expected a u64"))
                })?);
            }
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

/// The seed for this run: an explicit `--seed` wins, demo modes default to 42,
/// and live play falls back to the clock.
pub fn resolved_seed(args: &Args) -> u64 {
    match args.seed {
        Some(seed) => seed,
        None if args.demo || args.demo_merge => DEMO_SEED,
        None => default_seed(),
    }
}

/// Run a headless game and return the exact SPEC 5.6 dump text.
pub fn headless_dump(seed: u64, moves: &[Direction]) -> String {
    let store = MemoryStore::new();
    let mut game = Game::new(seed, store.load().best());
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
    fn parses_all_spec_flags() {
        let parsed = args(&[
            "--render-once",
            "--self-test-tui",
            "--no-persist",
            "--demo",
            "--demo-merge",
            "--fps",
            "60",
            "--frame",
            "14",
            "--seed",
            "42",
        ])
        .unwrap();

        assert_eq!(parsed.seed, Some(42));
        assert_eq!(parsed.fps, 60);
        assert_eq!(parsed.frame, Some(14));
        assert!(parsed.render_once);
        assert!(parsed.self_test_tui);
        assert!(parsed.no_persist);
        assert!(parsed.demo);
        assert!(parsed.demo_merge);
    }

    #[test]
    fn default_fps_is_60() {
        assert_eq!(DEFAULT_FPS, 60);
        assert!(
            USAGE.contains("default 60"),
            "usage must state the new default: {USAGE}"
        );
        assert_eq!(args(&[]).unwrap().fps, 60, "no --fps means 60");
        assert_eq!(args(&["--fps", "30"]).unwrap().fps, 30, "--fps 30 parses");
        assert!(args(&["--fps", "0"]).is_err(), "--fps 0 is still rejected");
    }

    #[test]
    fn rejects_zero_or_bad_fps() {
        assert!(args(&["--fps", "0"])
            .unwrap_err()
            .message()
            .contains("--fps"));
        assert!(args(&["--fps", "x"])
            .unwrap_err()
            .message()
            .contains("--fps"));
        assert!(args(&["--fps"]).is_err());
        assert_eq!(args(&["--fps", "60"]).unwrap().fps, 60);
        assert_eq!(args(&[]).unwrap().fps, DEFAULT_FPS);
    }

    #[test]
    fn rejects_bad_frame() {
        assert!(args(&["--frame", "x"])
            .unwrap_err()
            .message()
            .contains("--frame"));
        assert!(args(&["--frame"]).is_err());
        assert_eq!(args(&["--frame", "14"]).unwrap().frame, Some(14));
    }

    #[test]
    fn resolved_seed_prefers_explicit_then_demo_then_clock() {
        let explicit = args(&["--seed", "7", "--demo"]).unwrap();
        assert_eq!(resolved_seed(&explicit), 7);

        let demo = args(&["--demo"]).unwrap();
        assert_eq!(resolved_seed(&demo), DEMO_SEED);
        let demo_merge = args(&["--demo-merge"]).unwrap();
        assert_eq!(resolved_seed(&demo_merge), DEMO_SEED);

        let live = args(&[]).unwrap();
        assert_ne!(resolved_seed(&live), DEMO_SEED);
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
