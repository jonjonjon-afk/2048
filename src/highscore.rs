//! High-score storage behind one interface.
//!
//! The file format is one decimal `u64` and a newline. Missing or corrupt
//! files load as zero. Saves go to a sibling `.tmp` file and are renamed into
//! place, so a reader never observes a half-written score.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// File name used inside the application data directory.
pub const FILE_NAME: &str = "highscore";

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

/// The storage contract shared by the real file and the in-memory demo store.
pub trait HighScoreStore {
    /// Load the best score. A missing or corrupt file is zero, never an error.
    fn load(&self) -> u64;

    /// Persist the best score.
    fn save(&mut self, score: u64) -> Result<(), HighScoreError>;
}

/// The real store, backed by a single file.
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

    /// The default per-user path:
    /// `ProjectDirs::from("", "", "2048").data_dir()/highscore`.
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
    fn load(&self) -> u64 {
        let Ok(text) = fs::read_to_string(&self.path) else {
            return 0;
        };
        text.trim().parse::<u64>().unwrap_or(0)
    }

    fn save(&mut self, score: u64) -> Result<(), HighScoreError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let temporary = self.temp_path();
        fs::write(&temporary, format!("{score}\n"))?;
        fs::rename(&temporary, &self.path)?;
        Ok(())
    }
}

/// An in-memory store for demo, no-persist and headless dump modes.
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    score: u64,
}

impl MemoryStore {
    /// Create a memory store with an initial value.
    pub fn new(score: u64) -> Self {
        Self { score }
    }

    /// The currently held value.
    pub fn value(&self) -> u64 {
        self.score
    }
}

impl HighScoreStore for MemoryStore {
    fn load(&self) -> u64 {
        self.score
    }

    fn save(&mut self, score: u64) -> Result<(), HighScoreError> {
        self.score = score;
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

    fn round_trip<S: HighScoreStore>(store: &mut S) {
        assert_eq!(store.load(), 0);
        store.save(4096).expect("save 4096");
        assert_eq!(store.load(), 4096);
        store.save(8192).expect("save 8192");
        assert_eq!(store.load(), 8192);
    }

    #[test]
    fn file_store_round_trips_and_overwrites() {
        let dir = TempDir::new("round-trip");
        let mut store = FileStore::new(dir.join(FILE_NAME));

        store.save(4096).expect("save first score");
        assert_eq!(store.load(), 4096);
        store.save(8192).expect("overwrite score");
        assert_eq!(store.load(), 8192);
        assert_eq!(fs::read_to_string(store.path()).unwrap(), "8192\n");
    }

    #[test]
    fn missing_file_loads_as_zero() {
        let dir = TempDir::new("missing");
        let store = FileStore::new(dir.join(FILE_NAME));

        assert!(!store.path().exists());
        assert_eq!(store.load(), 0);
    }

    #[test]
    fn corrupt_file_loads_as_zero() {
        let dir = TempDir::new("corrupt");
        let path = dir.join(FILE_NAME);
        fs::write(&path, "oops\n").expect("write corrupt file");
        let store = FileStore::new(&path);

        assert_eq!(store.load(), 0);

        fs::write(&path, "").expect("write empty file");
        assert_eq!(store.load(), 0);
    }

    #[test]
    fn memory_store_implements_the_same_contract() {
        let mut store = MemoryStore::new(0);

        round_trip(&mut store);
        assert_eq!(store.value(), 8192);
    }

    #[test]
    fn save_uses_a_sibling_tmp_file_then_renames() {
        let dir = TempDir::new("atomic");
        let path = dir.join(FILE_NAME);
        let mut store = FileStore::new(&path);
        let temporary = store.temp_path();

        assert_eq!(temporary.parent(), path.parent());
        assert_eq!(
            temporary.file_name().and_then(|name| name.to_str()),
            Some("highscore.tmp")
        );
        assert!(!temporary.exists());

        store.save(4096).expect("save score");

        assert!(!temporary.exists());
        assert_eq!(fs::read_to_string(path).unwrap(), "4096\n");
    }
}
