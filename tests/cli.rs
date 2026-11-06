//! End-to-end checks against the real binary. Dump mode never touches storage.

use std::process::{Command, Output};

const BINARY: &str = env!("CARGO_BIN_EXE_game2048");

fn run(arguments: &[&str]) -> Output {
    Command::new(BINARY)
        .args(arguments)
        .output()
        .expect("run the game2048 binary")
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
fn help_exits_zero() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("Usage: game2048"));
    assert!(stdout.contains("--dump"));
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
