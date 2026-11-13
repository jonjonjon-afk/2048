//! Pure slide and jiggle geometry (SPEC 5.4, M13).
//!
//! This module is a set of functions over injected ages: no clock, no I/O and
//! no rendering. The renderer asks for an anchor or a jiggle offset and draws
//! exactly that.

use core::time::Duration;

use crate::game::Track;

/// Duration of one tile slide, in milliseconds (SPEC 5.4).
pub const SLIDE_MILLIS: u64 = 140;
/// Duration of one merged-card jiggle, in milliseconds (SPEC 5.4).
pub const JIGGLE_MILLIS: u64 = 240;
/// Length of one jiggle step, in milliseconds (SPEC 5.4).
pub const JIGGLE_STEP_MILLIS: u64 = 40;
/// Horizontal jiggle offsets, one per [`JIGGLE_STEP_MILLIS`] step.
pub const JIGGLE_OFFSETS: [i32; 6] = [2, -2, 1, -1, 0, 0];
/// Horizontal pitch of one board cell: a 9-wide card plus a 1-cell gap.
pub const CELL_PITCH_X: i32 = 10;
/// Vertical pitch of one board cell: a 5-tall card plus a 1-cell gap.
pub const CELL_PITCH_Y: i32 = 6;
/// Largest clamped card anchor column inside the 39x23 board.
pub const MAX_ANCHOR_X: i32 = 30;
/// Largest clamped card anchor row inside the 39x23 board.
pub const MAX_ANCHOR_Y: i32 = 18;

/// Ease-out quadratic progress for a slide at `age` (SPEC 5.4).
///
/// `x = age_ms / 140` clamped to `1.0`; `p = 1 - (1 - x)^2`.
pub fn progress(age: Duration) -> f64 {
    let x = (age.as_millis() as f64 / SLIDE_MILLIS as f64).clamp(0.0, 1.0);
    1.0 - (1.0 - x) * (1.0 - x)
}

/// A slide clock; [`Slide::anchor`] maps a track to its rounded card anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slide {
    /// Time since the move, advanced only by the caller's `dt`.
    pub age: Duration,
}

impl Slide {
    /// The card anchor `(x, y)` in play-area cells for `track` at this age.
    ///
    /// `x = round(from_x + (to_x - from_x) * p)` and the same for `y`, with
    /// `from_x = (from % 4) * 10`, `from_y = (from / 4) * 6` and the destination
    /// following the same formulas. `f64::round` rounds half away from zero.
    pub fn anchor(&self, track: &Track) -> (i32, i32) {
        let p = progress(self.age);
        let from_x = (track.from % 4) as i32 * CELL_PITCH_X;
        let from_y = (track.from / 4) as i32 * CELL_PITCH_Y;
        let to_x = (track.to % 4) as i32 * CELL_PITCH_X;
        let to_y = (track.to / 4) as i32 * CELL_PITCH_Y;
        let x = f64::from(from_x) + f64::from(to_x - from_x) * p;
        let y = f64::from(from_y) + f64::from(to_y - from_y) * p;

        (x.round() as i32, y.round() as i32)
    }
}

/// Horizontal jiggle offset at `age`, or `None` once the wobble has ended.
///
/// `k = age_ms / 40` indexes [`JIGGLE_OFFSETS`]; `age >= 240` clears it.
pub fn jiggle_dx(age: Duration) -> Option<i32> {
    if age >= Duration::from_millis(JIGGLE_MILLIS) {
        return None;
    }
    let step = (age.as_millis() / u128::from(JIGGLE_STEP_MILLIS)) as usize;
    Some(JIGGLE_OFFSETS[step])
}

/// Clamp a card anchor so a wall merge never draws outside the 39x23 board.
pub fn clamped_anchor(x: i32, y: i32) -> (i32, i32) {
    (x.clamp(0, MAX_ANCHOR_X), y.clamp(0, MAX_ANCHOR_Y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn millis(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    fn track(from: usize, to: usize, value: u32) -> Track {
        Track { from, to, value }
    }

    #[test]
    fn ease_out_progress_pins() {
        // G21: 0, 0.4375, 0.75, 0.9375, 1.0, then clamped at 1.0.
        for (age, expected) in [
            (0u64, 0.0f64),
            (35, 0.4375),
            (70, 0.75),
            (105, 0.9375),
            (140, 1.0),
            (200, 1.0),
        ] {
            assert_eq!(progress(millis(age)), expected, "{age} ms");
        }
        assert_eq!(SLIDE_MILLIS, 140);
    }

    #[test]
    fn slide_anchor_pins() {
        // G21: cell 0 -> cell 3 crosses 30 columns on row 0.
        let track = track(0, 3, 2);
        for (age, expected) in [
            (0u64, (0, 0)),
            (35, (13, 0)),
            (70, (23, 0)),
            (105, (28, 0)),
            (140, (30, 0)),
        ] {
            let slide = Slide { age: millis(age) };
            assert_eq!(slide.anchor(&track), expected, "{age} ms");
        }
    }

    #[test]
    fn vertical_slide_anchor_pins() {
        // Cell 12 (row 3) -> cell 0 (row 0): y 18 -> 0, and 18 - 13.5 = 4.5
        // rounds half away from zero to 5 at 70 ms.
        let track = track(12, 0, 2);
        let at_zero = Slide {
            age: Duration::ZERO,
        };
        assert_eq!(at_zero.anchor(&track), (0, 18));
        let at_70 = Slide { age: millis(70) };
        assert_eq!(at_70.anchor(&track), (0, 5));
    }

    #[test]
    fn jiggle_offsets_pin_to_the_table() {
        // G22: k = age / 40 with the age < 240 guard.
        for (age, expected) in [(0u64, 2), (40, -2), (80, 1), (120, -1), (160, 0), (200, 0)] {
            assert_eq!(jiggle_dx(millis(age)), Some(expected), "{age} ms");
        }
        assert_eq!(JIGGLE_OFFSETS, [2, -2, 1, -1, 0, 0]);
    }

    #[test]
    fn jiggle_offsets_stop_at_240_ms() {
        assert_eq!(jiggle_dx(millis(239)), Some(0), "the last step still runs");
        assert_eq!(jiggle_dx(millis(240)), None);
        assert_eq!(jiggle_dx(millis(1_000)), None);
        assert_eq!(JIGGLE_MILLIS, 240);
    }

    #[test]
    fn anchors_are_clamped_to_the_board() {
        // A cell-0 merge at age 40 has dx = -2: slot 0 - 2 clamps to 0.
        assert_eq!(jiggle_dx(millis(40)), Some(-2));
        assert_eq!(clamped_anchor(0 - 2, 0), (0, 0));
        // A cell-3 merge at age 0 has dx = +2: slot 30 + 2 clamps to 30.
        assert_eq!(jiggle_dx(Duration::ZERO), Some(2));
        assert_eq!(clamped_anchor(30 + 2, 18), (30, 18));
        // Any out-of-range pair clamps to the documented board limits.
        assert_eq!(clamped_anchor(99, 99), (MAX_ANCHOR_X, MAX_ANCHOR_Y));
        assert_eq!(clamped_anchor(-99, -99), (0, 0));
    }
}
