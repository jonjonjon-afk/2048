//! Score count-up: the displayed score that chases the real one.
//!
//! This is display state only. It never changes the board, the score, the best
//! score or the RNG, and it advances by an injected `dt` so tests can assert
//! exact values. The panel and the game-over overlay both read it through
//! [`crate::update::App::shown_score`].

use core::time::Duration;

/// Time a score change takes to reach its new value, in milliseconds.
pub const SCORE_COUNT_MILLIS: u64 = 450;

/// How long one count takes.
const COUNT: Duration = Duration::from_millis(SCORE_COUNT_MILLIS);

/// One score count from `from` towards `to`, driven by the ease-out of SPEC §5.7.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreCount {
    from: u64,
    to: u64,
    elapsed: Duration,
}

impl ScoreCount {
    /// A count that already shows `value` and is not animating.
    pub const fn new(value: u64) -> Self {
        Self {
            from: value,
            to: value,
            elapsed: Duration::ZERO,
        }
    }

    /// The value the count started from.
    pub fn from(&self) -> u64 {
        self.from
    }

    /// The real score the count is heading to.
    pub fn target(&self) -> u64 {
        self.to
    }

    /// Time since the current count started.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// The value to display right now.
    ///
    /// `elapsed >= 450` shows the target exactly. Otherwise `progress` is the
    /// cubic ease-out `1 - (1 - t/450)^3` and the displayed value is
    /// `from + floor((to - from) * progress)`. A target at or below the start
    /// (only reachable from a restart or a test fixture) shows at once.
    pub fn shown(&self) -> u64 {
        if self.to <= self.from || self.elapsed >= COUNT {
            return self.to;
        }

        let fraction = self.elapsed.as_secs_f64() / COUNT.as_secs_f64();
        let progress = 1.0 - (1.0 - fraction).powi(3);
        let delta = self.to - self.from;
        let counted = (delta as f64 * progress).floor() as u64;
        self.from + counted.min(delta)
    }

    /// Aim at `target` from the value currently shown, restarting the clock.
    ///
    /// Setting the same target again changes nothing, so an update that does
    /// not score keeps the running count intact.
    pub fn retarget(&mut self, target: u64) {
        if target == self.to {
            return;
        }
        self.from = self.shown();
        self.to = target;
        self.elapsed = Duration::ZERO;
    }

    /// Show `value` immediately with no counting.
    pub fn reset(&mut self, value: u64) {
        *self = Self::new(value);
    }

    /// Advance the count by `dt`; the value never runs past its target.
    pub fn advance(&mut self, dt: Duration) {
        if self.to <= self.from {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(dt).min(COUNT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn millis(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn score_count_follows_the_g14_curve() {
        let mut count = ScoreCount::new(0);
        count.retarget(16);

        // G14: floor(16 * (1 - (1 - t/450)^3)) at t = 0, 50, ..., 450 ms.
        let expected = [0u64, 4, 8, 11, 13, 14, 15, 15, 15, 16];
        assert_eq!(count.shown(), expected[0], "at t = 0 ms");

        for (step, want) in expected.iter().enumerate().skip(1) {
            count.advance(millis(50));
            assert_eq!(count.shown(), *want, "at t = {} ms", step * 50);
        }
    }

    #[test]
    fn score_count_clamps_to_the_target_at_450ms() {
        let mut count = ScoreCount::new(0);
        count.retarget(16);

        count.advance(millis(449));
        assert_eq!(count.shown(), 15, "at t = 449 ms");

        count.advance(millis(1));
        assert_eq!(count.shown(), 16, "at t = 450 ms");

        count.advance(millis(450));
        assert_eq!(count.shown(), 16, "at t = 900 ms");
        assert_eq!(count.target(), 16);
    }
}
