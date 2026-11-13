//! A3: a named table entry survives a real process restart through the default path.
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

const CHILD_ENTRY_ENV: &str = "GAME2048_RESTART_CHILD_ENTRY";
const TEST_NAME: &str = "named_entry_survives_a_process_restart";

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

/// Run the test binary again as a child that inserts `name`/`score` through the
/// default path, and return the path it resolved.
fn run_child(xdg: &Path, name: &str, score: u64) -> PathBuf {
    let exe = env::current_exe().expect("locate the running test binary");
    let output = Command::new(exe)
        .args(["--exact", TEST_NAME, "--nocapture"])
        .env(CHILD_ENTRY_ENV, format!("{name}:{score}"))
        .env("XDG_DATA_HOME", xdg)
        .output()
        .expect("spawn the child test process");

    assert!(
        output.status.success(),
        "child insert of {name} {score} failed:\n{}",
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
fn named_entry_survives_a_process_restart() {
    // Child mode: a separate process adds one named entry through the real
    // default path and saves the whole table.
    if let Ok(entry) = env::var(CHILD_ENTRY_ENV) {
        let (name, score) = entry
            .split_once(':')
            .expect("the child entry is NAME:SCORE");
        let score: u64 = score.parse().expect("the child score is a u64");
        let mut store = FileStore::from_default_path().expect("the child resolves a default path");
        let mut table = store.load();
        table.insert(name, score);
        println!("resolved_path={}", store.path().display());
        store.save(&table).expect("the child saves the table");
        return;
    }

    // Parent mode: nothing here changes the parent's environment.
    let xdg_before = env::var_os("XDG_DATA_HOME");
    let dir = TempDir::new();
    let expected = dir.path.join("2048").join(FILE_NAME);
    assert_eq!(
        expected,
        dir.path.join("2048").join("highscores"),
        "the default path is $XDG_DATA_HOME/2048/highscores"
    );

    let first = run_child(&dir.path, "ALICE", 4096);
    assert_eq!(
        first, expected,
        "child 1 must resolve $XDG_DATA_HOME/2048/highscores"
    );
    assert!(
        first.starts_with(&dir.path),
        "the table must live under the temp XDG dir, never $HOME: {}",
        first.display()
    );

    let table = FileStore::new(&first).load();
    assert_eq!(table.best(), 4096);
    assert_eq!(table.entries().len(), 1);
    assert_eq!(table.entries()[0].name, "ALICE");
    assert_eq!(
        fs::read_to_string(&expected).expect("read the default table"),
        "ALICE 4096\n"
    );
    println!(
        "child 1 resolved {} and the parent loaded ALICE 4096",
        first.display()
    );

    let second = run_child(&dir.path, "BOB", 8192);
    assert_eq!(
        second, expected,
        "child 2 must resolve the same default path"
    );

    let table = FileStore::new(&second).load();
    assert_eq!(table.best(), 8192);
    let rows: Vec<(String, u64)> = table
        .entries()
        .iter()
        .map(|entry| (entry.name.clone(), entry.score))
        .collect();
    assert_eq!(
        rows,
        vec![("BOB".to_owned(), 8192), ("ALICE".to_owned(), 4096)]
    );
    assert_eq!(
        fs::read_to_string(&expected).expect("read the default table"),
        "BOB 8192\nALICE 4096\n"
    );
    println!(
        "child 2 resolved {} and the parent loaded BOB 8192, ALICE 4096",
        second.display()
    );

    // The parent's environment is untouched.
    assert_eq!(
        env::var_os("XDG_DATA_HOME"),
        xdg_before,
        "the parent's XDG_DATA_HOME must not be mutated"
    );

    let removed = dir.path.clone();
    drop(dir);
    assert!(
        !removed.exists(),
        "the test must leave no temp directory behind"
    );
    println!("temp dir {} removed", removed.display());
}
