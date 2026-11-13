//! End-to-end checks against the real binary. Dump and render modes never
//! touch storage; the persistence test uses an isolated `XDG_DATA_HOME`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_game2048");

fn run(arguments: &[&str]) -> Output {
    Command::new(BINARY)
        .args(arguments)
        .output()
        .expect("run the game2048 binary")
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after the epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "game2048-cli-{}-{nanos}-{label}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn dump_seed_42_matches_the_spec_block() {
    let output = run(&["--dump", "--seed", "42"]);

    assert!(output.status.success(), "dump must exit 0");
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        include_str!("fixtures/dump_seed42.txt")
    );
}

#[test]
fn dump_prints_the_real_score() {
    let output = run(&["--dump", "--seed", "42", "--moves", "lrud"]);

    assert!(output.status.success(), "dump must exit 0");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(
        stdout.contains("score: 4\n"),
        "dump must print the real score, never the counted one: {stdout}"
    );

    // The panel at the same moment counts from 0, so it shows less than the
    // dump. That is exactly why the dump stays on `game.score`.
    let frame = run(&[
        "--render-once",
        "--demo-merge",
        "--frame",
        "0",
        "--no-persist",
    ]);
    assert!(frame.status.success());
    let rendered = String::from_utf8(frame.stdout).expect("stdout is UTF-8");
    assert!(
        rendered.contains("SCORE 0"),
        "frame 0 of the demo merge shows the counted score 0"
    );
}

#[test]
fn help_exits_zero() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("Usage: game2048"));
    assert!(stdout.contains("--dump"));
    assert!(
        stdout.contains("default 60"),
        "--help must state the 60 FPS default"
    );
}

#[test]
fn unknown_flag_exits_two_with_stderr() {
    let output = run(&["--bogus"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("unknown argument '--bogus'"));
}

#[test]
fn bad_moves_character_exits_two_with_stderr() {
    let output = run(&["--dump", "--moves", "lx"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("invalid move 'x'"));
}

#[test]
fn moves_lrud_apply_in_order() {
    let output = run(&["--dump", "--seed", "42", "--moves", "lrud"]);

    assert!(output.status.success());
    let expected = "\
board:
0 0 0 0
0 0 0 0
0 0 0 4
2 2 0 2
score: 4
best: 4
won: false
game_over: false
";
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        expected
    );
}

#[test]
fn render_once_prints_36_lines_of_100_columns_without_ansi() {
    let output = run(&["--render-once", "--seed", "42", "--no-persist"]);

    assert!(output.status.success(), "render-once must exit 0");
    assert!(
        !output.stdout.contains(&0x1b),
        "render-once must print no ANSI escapes"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.ends_with('\n'));

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 36, "exactly 36 lines");
    assert!(
        lines.iter().all(|line| line.chars().count() == 100),
        "every line is exactly 100 characters"
    );
    for needle in ["2048", "SCORE", "BEST"] {
        assert!(stdout.contains(needle), "frame is missing {needle}");
    }
}

#[test]
fn bad_fps_and_bad_frame_exit_two_with_stderr() {
    for arguments in [vec!["--fps", "0"], vec!["--fps", "x"], vec!["--frame", "x"]] {
        let output = run(&arguments);
        assert_eq!(output.status.code(), Some(2), "arguments: {arguments:?}");
        let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
        assert!(
            stderr.contains("invalid --fps") || stderr.contains("invalid --frame"),
            "arguments {arguments:?} printed: {stderr}"
        );
    }
}

#[test]
fn no_persist_never_writes_the_real_path() {
    let xdg = temp_dir("no-persist");
    let output = Command::new(BINARY)
        .args(["--render-once", "--seed", "42", "--no-persist"])
        .env("XDG_DATA_HOME", &xdg)
        .output()
        .expect("run the game2048 binary");

    assert!(output.status.success());
    assert!(
        !xdg.join("2048").exists(),
        "--no-persist must not create the data directory"
    );
    assert!(!xdg.join("2048").join("highscores").exists());
    assert!(!xdg.join("2048").join("highscore").exists());

    let _ = fs::remove_dir_all(&xdg);
}

#[test]
fn demo_and_demo_merge_render_the_same_frame() {
    let demo = run(&["--render-once", "--demo", "--frame", "14", "--no-persist"]);
    let demo_merge = run(&[
        "--render-once",
        "--demo-merge",
        "--frame",
        "14",
        "--no-persist",
    ]);

    assert!(demo.status.success(), "--demo must exit 0");
    assert!(demo_merge.status.success(), "--demo-merge must exit 0");
    assert_eq!(demo.stdout, demo_merge.stdout);
}

#[test]
fn render_once_demo_merge_frame14_contains_ripple_and_spark() {
    let output = run(&[
        "--render-once",
        "--demo-merge",
        "--frame",
        "14",
        "--no-persist",
    ]);

    assert!(output.status.success(), "--demo-merge must exit 0");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains('▀'), "frame 14 is missing the ▀ ripple");
    assert!(stdout.contains('▄'), "frame 14 is missing the ▄ ripple");
    assert!(stdout.contains('✦'), "frame 14 is missing the ✦ spark");
}

#[test]
fn effect_snapshots_match_frames_4_14_24() {
    let fixtures = [
        (4u64, include_str!("fixtures/effect_100x36_frame_04.txt")),
        (14, include_str!("fixtures/effect_100x36_frame_14.txt")),
        (24, include_str!("fixtures/effect_100x36_frame_24.txt")),
    ];

    for (frame, fixture) in fixtures {
        let number = frame.to_string();
        let output = run(&[
            "--render-once",
            "--demo-merge",
            "--frame",
            &number,
            "--no-persist",
        ]);

        assert!(output.status.success(), "frame {frame} must exit 0");
        assert_eq!(
            String::from_utf8(output.stdout).expect("stdout is UTF-8"),
            fixture,
            "frame {frame} must match its committed fixture"
        );
    }
}
