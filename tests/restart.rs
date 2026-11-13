//! A3: the best score survives a real process restart through the default path.
//!
//! The test binary re-executes itself (`std::env::current_exe()`), so each
//! writer is a genuinely separate process. Only the child sees the temporary
//! `XDG_DATA_HOME`; the parent's own environment is never mutated.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use game2048::highscore::{FileStore, HighScoreStore, FILE_NAME};

const CHILD_SCORE_ENV: &str = "GAME2048_RESTART_CHILD_SCORE";
const TEST_NAME: &str = "highscore_survives_a_process_restart";

/// An explicit temp directory; no test reads the real HOME or data path.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = env::temp_dir().join(format!(
            "game2048-restart-{}-{}-{}",
            std::process::id(),
            nanos,
            id
        ));
        fs::create_dir_all(&path).expect("create the isolated temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Run the test binary again as a child that saves `score` through the default path.
fn run_child(xdg: &Path, score: u64) -> PathBuf {
    let exe = env::current_exe().expect("locate the running test binary");
    let output = Command::new(exe)
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(CHILD_SCORE_ENV, score.to_string())
        .env("XDG_DATA_HOME", xdg)
        .output()
        .expect("spawn the child test process");

    assert!(
        output.status.success(),
        "child save of {score} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("child stdout is UTF-8");
    let line = stdout
        .lines()
        .find(|line| line.starts_with("resolved_path="))
        .unwrap_or_else(|| panic!("child did not report its resolved path:\n{stdout}"));
    PathBuf::from(line.trim_start_matches("resolved_path="))
}

#[test]
fn highscore_survives_a_process_restart() {
    // Child mode: a separate process saves one score through the real default path.
    if let Ok(score) = env::var(CHILD_SCORE_ENV) {
        let score: u64 = score.parse().expect("child score is a u64");
        let mut store = FileStore::from_default_path().expect("the child resolves a default path");
        println!("resolved_path={}", store.path().display());
        store.save(score).expect("the child saves the score");
        return;
    }

    // Parent mode: nothing here changes the parent's environment.
    let dir = TempDir::new();
    let expected = dir.path.join("2048").join(FILE_NAME);
    assert_eq!(
        expected,
        dir.path.join("2048").join("highscore"),
        "the default path is $XDG_DATA_HOME/2048/highscore"
    );

    let first = run_child(&dir.path, 4096);
    assert_eq!(
        first, expected,
        "child 1 must resolve $XDG_DATA_HOME/2048/highscore"
    );
    assert!(
        first.starts_with(&dir.path),
        "the score must live under the temp XDG dir, never $HOME: {}",
        first.display()
    );
    assert!(
        dir.path.starts_with(env::temp_dir()),
        "the isolated XDG dir itself lives under the system temp dir"
    );

    // A fresh store in the parent sees what the first child wrote.
    let reader = FileStore::new(first.as_path());
    assert_eq!(reader.load(), 4096, "the parent must load child 1's score");
    println!(
        "child 1 resolved {} and the parent loaded 4096",
        first.display()
    );

    let second = run_child(&dir.path, 8192);
    assert_eq!(
        second, expected,
        "child 2 must resolve the same default path"
    );

    // Another fresh store sees the overwritten score.
    let reader = FileStore::new(second.as_path());
    assert_eq!(reader.load(), 8192, "the parent must load child 2's score");
    assert_eq!(
        fs::read_to_string(&expected).expect("read the default high score file"),
        "8192\n"
    );
    println!(
        "child 2 resolved {} and the parent loaded 8192",
        second.display()
    );

    let removed = dir.path.clone();
    drop(dir);
    assert!(
        !removed.exists(),
        "the test must leave no temp directory behind"
    );
    println!("temp dir {} removed", removed.display());
}
