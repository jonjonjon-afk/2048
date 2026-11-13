//! Named high-score table storage behind one interface.
//!
//! The table file `highscores` holds one `NAME SCORE` line per entry, sorted by
//! score descending, at most ten entries. A missing or unreadable file is an
//! empty table; a malformed line is skipped. Writes go to a sibling `.tmp` file
//! and are renamed into place, so a reader never observes a half-written table.
//!
//! The legacy single-score file `highscore` is read once for migration only and
//! is never written again.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// File name of the table inside the application data directory.
pub const FILE_NAME: &str = "highscores";
/// File name of the legacy single-score file, read once for migration.
pub const LEGACY_FILE_NAME: &str = "highscore";
/// Maximum number of entries kept in the table.
pub const MAX_ENTRIES: usize = 10;
/// Maximum length of a name, in characters.
pub const NAME_MAX_LEN: usize = 10;

/// Errors from the file-backed store.
#[derive(Debug)]
pub enum HighScoreError {
    /// The filesystem rejected a read or write.
    Io(io::Error),
    /// `ProjectDirs` could not determine a data directory.
    NoDefaultPath,
}

impl fmt::Display for HighScoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "high-score storage failed: {error}"),
            Self::NoDefaultPath => write!(formatter, "no default data directory is available"),
        }
    }
}

impl std::error::Error for HighScoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::NoDefaultPath => None,
        }
    }
}

impl From<io::Error> for HighScoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// One named score in the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u64,
}

/// The named table: at most ten entries, best first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HighScores {
    entries: Vec<ScoreEntry>,
}

impl HighScores {
    /// An empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// The rows, best first.
    pub fn entries(&self) -> &[ScoreEntry] {
        &self.entries
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the table holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The top score, or 0 for an empty table (SPEC §5.3).
    pub fn best(&self) -> u64 {
        self.entries.first().map_or(0, |entry| entry.score)
    }

    /// True when `score` earns a place (G17 / SPEC §5.3).
    pub fn qualifies(&self, score: u64) -> bool {
        score > 0
            && (self.entries.len() < MAX_ENTRIES
                || self.entries.last().is_some_and(|entry| score > entry.score))
    }

    /// Insert a score at its sorted position (new ties go after equals),
    /// then truncate to ten entries.
    pub fn insert(&mut self, name: impl Into<String>, score: u64) {
        let position = self
            .entries
            .iter()
            .position(|entry| entry.score < score)
            .unwrap_or(self.entries.len());
        self.entries.insert(
            position,
            ScoreEntry {
                name: name.into(),
                score,
            },
        );
        self.entries.truncate(MAX_ENTRIES);
    }

    /// Parse the file format, skipping malformed lines.
    ///
    /// The parser also accepts `-` in a name so a migrated legacy score can
    /// live as `---`. Loaded rows are put back into score order and capped at
    /// ten, which keeps a hand-edited file harmless.
    pub fn parse(text: &str) -> Self {
        let mut entries = Vec::new();
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let (Some(name), Some(score), None) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            if !valid_name(name) {
                continue;
            }
            let Ok(score) = score.parse::<u64>() else {
                continue;
            };
            entries.push(ScoreEntry {
                name: name.to_owned(),
                score,
            });
        }
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.score));
        entries.truncate(MAX_ENTRIES);
        Self { entries }
    }

    /// Serialise exactly as the file expects: one `NAME SCORE` line each.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for entry in &self.entries {
            text.push_str(&format!("{} {}\n", entry.name, entry.score));
        }
        text
    }

    /// The one-row table used for the legacy migration (`--- v`).
    fn single(name: &str, score: u64) -> Self {
        Self {
            entries: vec![ScoreEntry {
                name: name.to_owned(),
                score,
            }],
        }
    }
}

/// A name the parser accepts: 1-10 characters from `A-Z`, `0-9` and `-`.
fn valid_name(name: &str) -> bool {
    let length = name.chars().count();
    (1..=NAME_MAX_LEN).contains(&length)
        && name.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
        })
}

/// The storage contract shared by the real file and the in-memory demo store.
pub trait HighScoreStore {
    /// Load the table. A missing or unreadable file is empty, never an error.
    fn load(&self) -> HighScores;

    /// Persist the whole table.
    fn save(&mut self, table: &HighScores) -> Result<(), HighScoreError>;
}

/// The real store, backed by the `highscores` file.
#[derive(Clone, Debug)]
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// Create a store for an explicit path. Tests always pass a temp path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The path this store reads and writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The sibling temporary file used for atomic replacement.
    pub fn temp_path(&self) -> PathBuf {
        let name = self
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| FILE_NAME.to_owned());
        self.path.with_file_name(format!("{name}.tmp"))
    }

    /// The legacy single-score file next to the table.
    pub fn legacy_path(&self) -> PathBuf {
        self.path.with_file_name(LEGACY_FILE_NAME)
    }

    /// The default per-user table path:
    /// `ProjectDirs::from("", "", "2048").data_dir()/highscores`.
    pub fn default_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "2048").map(|dirs| dirs.data_dir().join(FILE_NAME))
    }

    /// Open the default per-user store.
    pub fn from_default_path() -> Result<Self, HighScoreError> {
        Self::default_path()
            .map(Self::new)
            .ok_or(HighScoreError::NoDefaultPath)
    }
}

impl HighScoreStore for FileStore {
    fn load(&self) -> HighScores {
        if let Ok(text) = fs::read_to_string(&self.path) {
            return HighScores::parse(&text);
        }

        // Migration only: the table is missing, so a positive legacy score
        // becomes the single `---` entry. Nothing is written here.
        let Ok(text) = fs::read_to_string(self.legacy_path()) else {
            return HighScores::new();
        };
        match text.trim().parse::<u64>() {
            Ok(score) if score > 0 => HighScores::single("---", score),
            _ => HighScores::new(),
        }
    }

    fn save(&mut self, table: &HighScores) -> Result<(), HighScoreError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let temporary = self.temp_path();
        fs::write(&temporary, table.to_text())?;
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }
}

/// An in-memory store for demo, no-persist and headless dump modes.
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    table: HighScores,
}

impl MemoryStore {
    /// An empty in-memory table.
    pub fn new() -> Self {
        Self::default()
    }

    /// An in-memory store seeded with an explicit table.
    pub fn with_table(table: HighScores) -> Self {
        Self { table }
    }

    /// The table currently held.
    pub fn table(&self) -> &HighScores {
        &self.table
    }
}

impl HighScoreStore for MemoryStore {
    fn load(&self) -> HighScores {
        self.table.clone()
    }

    fn save(&mut self, table: &HighScores) -> Result<(), HighScoreError> {
        self.table = table.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    /// An explicit temp directory; no test reads the real HOME or data path.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!(
                "game2048-test-{}-{}-{}-{}",
                std::process::id(),
                nanos,
                id,
                label
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn join(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

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

    fn pair(name: &str, score: u64) -> (String, u64) {
        (name.to_owned(), score)
    }

    #[test]
    fn table_insert_sorts_descending_and_keeps_ties_after_equals() {
        // G16: AAA 100, BBB 100, CCC 50, then DDD 200.
        let table = table(&[("AAA", 100), ("BBB", 100), ("CCC", 50), ("DDD", 200)]);

        assert_eq!(
            rows(&table),
            vec![
                pair("DDD", 200),
                pair("AAA", 100),
                pair("BBB", 100),
                pair("CCC", 50)
            ]
        );
        assert_eq!(table.len(), 4);
        assert_eq!(table.best(), 200);
    }

    #[test]
    fn table_is_truncated_to_ten() {
        let mut table = table(&[
            ("AAA", 100),
            ("BBB", 90),
            ("CCC", 80),
            ("DDD", 70),
            ("EEE", 60),
            ("FFF", 50),
            ("GGG", 40),
            ("HHH", 30),
            ("III", 20),
            ("JJJ", 10),
        ]);
        assert_eq!(table.len(), MAX_ENTRIES);

        table.insert("LOW", 5);
        assert_eq!(table.len(), 10);
        assert!(
            !rows(&table).contains(&pair("LOW", 5)),
            "an 11th entry below the 10th must be dropped"
        );
        assert_eq!(table.entries().last().map(|entry| entry.score), Some(10));
    }

    #[test]
    fn malformed_lines_are_skipped_and_a_missing_file_is_empty() {
        let dir = TempDir::new("malformed");
        let path = dir.join(FILE_NAME);
        fs::write(&path, "AAA 100\ngarbage\nBBB nope\nCCC 50\n").expect("write the table");
        let store = FileStore::new(&path);

        assert_eq!(rows(&store.load()), vec![pair("AAA", 100), pair("CCC", 50)]);

        let missing = FileStore::new(dir.join("absent"));
        let empty = missing.load();
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.best(), 0);
        assert!(!dir.join("absent").exists(), "load must not create a file");
    }

    #[test]
    fn table_parser_rejects_bad_names_and_accepts_dashes() {
        let parsed = HighScores::parse("--- 4096\nTOOLONGNAME1 5\nlower 5\n");
        assert_eq!(rows(&parsed), vec![pair("---", 4096)]);

        assert!(HighScores::parse("TOOLONGNAME1 5\n").is_empty());
        assert!(HighScores::parse("lower 5\n").is_empty());
        assert!(HighScores::parse("ABC 5\nDEF\n").len() == 1);
    }

    #[test]
    fn table_save_writes_the_expected_text_and_uses_a_tmp_sibling() {
        let dir = TempDir::new("save");
        let path = dir.join(FILE_NAME);
        let mut store = FileStore::new(&path);
        let temporary = store.temp_path();
        assert_eq!(temporary, dir.join("highscores.tmp"));
        assert!(!temporary.exists());

        // G16 table -> the exact file text.
        let mut table = HighScores::new();
        table.insert("DDD", 200);
        table.insert("AAA", 100);
        table.insert("BBB", 100);
        table.insert("CCC", 50);
        store.save(&table).expect("save the table");

        assert_eq!(
            fs::read_to_string(&path).expect("read the table"),
            "DDD 200\nAAA 100\nBBB 100\nCCC 50\n"
        );
        assert!(!temporary.exists(), "no .tmp may survive the rename");
    }

    #[test]
    fn legacy_highscore_becomes_a_dash_entry() {
        let dir = TempDir::new("legacy-become");
        fs::write(dir.join(LEGACY_FILE_NAME), "4096\n").expect("write the legacy score");
        let store = FileStore::new(dir.join(FILE_NAME));

        let table = store.load();
        assert_eq!(rows(&table), vec![pair("---", 4096)]);
        assert_eq!(table.best(), 4096);

        assert!(
            !dir.join(FILE_NAME).exists(),
            "loading must not write the table"
        );
        assert_eq!(rows(&store.load()), vec![pair("---", 4096)]);
        assert!(
            !dir.join(FILE_NAME).exists(),
            "a second load still writes nothing"
        );
    }

    #[test]
    fn legacy_highscore_is_ignored_when_the_table_exists() {
        let dir = TempDir::new("legacy-ignored");
        fs::write(dir.join(LEGACY_FILE_NAME), "4096\n").expect("write the legacy score");
        fs::write(dir.join(FILE_NAME), "BOB 100\n").expect("write the table");
        let store = FileStore::new(dir.join(FILE_NAME));

        assert_eq!(rows(&store.load()), vec![pair("BOB", 100)]);
    }

    #[test]
    fn legacy_file_is_never_written_and_migrates_on_the_next_save() {
        let dir = TempDir::new("legacy-save");
        fs::write(dir.join(LEGACY_FILE_NAME), "4096\n").expect("write the legacy score");
        let mut store = FileStore::new(dir.join(FILE_NAME));

        let mut table = store.load();
        assert_eq!(rows(&table), vec![pair("---", 4096)]);

        table.insert("ALICE", 8192);
        store.save(&table).expect("save the migrated table");

        assert_eq!(
            fs::read_to_string(dir.join(FILE_NAME)).expect("read the table"),
            "ALICE 8192\n--- 4096\n"
        );
        assert_eq!(
            fs::read_to_string(dir.join(LEGACY_FILE_NAME)).expect("read the legacy file"),
            "4096\n",
            "the legacy file must never be written again"
        );
    }

    #[test]
    fn memory_store_implements_the_same_contract() {
        let mut store = MemoryStore::new();
        assert_eq!(store.load().len(), 0);

        let table = table(&[("AAA", 100), ("BBB", 200)]);
        store.save(&table).expect("save into memory");

        assert_eq!(
            rows(&store.load()),
            vec![pair("BBB", 200), pair("AAA", 100)]
        );
        assert_eq!(store.table(), &table);

        let seeded = MemoryStore::with_table(table.clone());
        assert_eq!(seeded.load(), table);
    }

    #[test]
    fn table_round_trips_through_a_file() {
        let dir = TempDir::new("round-trip");
        let mut store = FileStore::new(dir.join(FILE_NAME));
        let table = table(&[("AAA", 100), ("BBB", 200)]);

        store.save(&table).expect("save the table");
        assert_eq!(store.load(), table);
        store.save(&HighScores::new()).expect("save an empty table");
        assert_eq!(store.load(), HighScores::new());
        assert_eq!(
            fs::read_to_string(store.path()).expect("read the table"),
            ""
        );
    }
}
