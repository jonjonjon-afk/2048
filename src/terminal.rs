//! Terminal ownership.
//!
//! One guard owns the terminal state. It restores raw mode and the alternate
//! screen on a normal drop and while a panic unwinds. The restore closure is
//! injectable so the lifecycle can be tested without a real terminal; the
//! live shell passes `ratatui::restore`, whose `ratatui::init()` counterpart
//! installs the panic hook.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Restores the terminal exactly once.
pub struct TerminalGuard {
    restore: Option<Box<dyn FnMut() + Send>>,
    restores: Arc<AtomicUsize>,
    restored: bool,
}

impl TerminalGuard {
    /// Create a guard around a restore function.
    pub fn new(restore: impl FnMut() + Send + 'static) -> Self {
        Self {
            restore: Some(Box::new(restore)),
            restores: Arc::new(AtomicUsize::new(0)),
            restored: false,
        }
    }

    /// A handle to the restore counter, usable after the guard is gone.
    pub fn counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.restores)
    }

    fn restore_now(&mut self) {
        if self.restored {
            return;
        }
        self.restored = true;
        if let Some(mut restore) = self.restore.take() {
            restore();
        }
        self.restores.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore_now();
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn terminal_guard_restores_on_normal_drop() {
        let guard = TerminalGuard::new(|| {});
        let counter = guard.counter();

        drop(guard);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn terminal_guard_restores_while_unwinding_a_panic() {
        let observed: Arc<Mutex<Option<Arc<AtomicUsize>>>> = Arc::new(Mutex::new(None));
        let slot = Arc::clone(&observed);

        let result = catch_unwind(AssertUnwindSafe(move || {
            let guard = TerminalGuard::new(|| {});
            *slot.lock().unwrap() = Some(guard.counter());
            panic!("simulated panic inside the terminal session");
        }));

        assert!(result.is_err(), "the simulated panic must unwind");
        let counter = observed
            .lock()
            .unwrap()
            .clone()
            .expect("the guard registered its counter");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_restore_runs_only_once_per_guard() {
        let calls = Arc::new(AtomicUsize::new(0));
        let hits = Arc::clone(&calls);
        let guard = TerminalGuard::new(move || {
            hits.fetch_add(1, Ordering::SeqCst);
        });
        let counter = guard.counter();

        drop(guard);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
