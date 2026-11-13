//! Deterministic CPU-rendered effects: merge ripples, sparks, flashes and the best pulse.
//!
//! The module is pure: time arrives through [`Effects::advance`] as an injected
//! `dt`, and the only randomness is a private effect RNG. The game rules and the
//! game RNG are never touched here.

use core::f32::consts::TAU;
use core::time::Duration;

use crate::game::Merge;
use crate::rng::Xorshift64;

/// Merge ripple lifetime, in milliseconds.
pub const RIPPLE_MILLIS: u64 = 900;
/// Ripple speed in play-area cells per millisecond (40 cells per second).
pub const RIPPLE_CELLS_PER_MILLI: f64 = 0.04;
/// Half-width of the ripple front, in cells (SPEC 5.4 `±0.75`).
pub const RIPPLE_FRONT_WINDOW: f64 = 0.75;
/// Amplitude factor applied for each play-area wall an image is mirrored across.
pub const RIPPLE_WALL_FACTOR: f64 = 0.55;
/// Ripple glyphs, strongest first, with the minimum amplitude for each.
pub const RIPPLE_THRESHOLDS: [(char, f64); 3] = [('▓', 0.55), ('▒', 0.30), ('░', 0.10)];
/// The three ripple glyphs, strongest first.
pub const RIPPLE_SYMBOLS: [char; 3] = ['▓', '▒', '░'];
/// Play-area width in cells (`x` in `0..=38`).
pub const PLAY_WIDTH: f64 = 39.0;
/// Play-area height in cells (`y` in `0..=22`).
pub const PLAY_HEIGHT: f64 = 23.0;
/// Play-area left/top wall coordinate.
pub const PLAY_WALL_START: f64 = -0.5;
/// Play-area right wall coordinate.
pub const PLAY_WALL_RIGHT: f64 = 38.5;
/// Play-area bottom wall coordinate.
pub const PLAY_WALL_BOTTOM: f64 = 22.5;
/// Horizontal pitch of one board cell: a 9-wide card plus a 1-cell gap.
pub const CELL_PITCH_X: f64 = 10.0;
/// Vertical pitch of one board cell: a 5-tall card plus a 1-cell gap.
pub const CELL_PITCH_Y: f64 = 6.0;
/// Offset from a cell's top-left corner to the merged card's centre, x.
pub const CELL_CENTRE_X: f64 = 4.5;
/// Offset from a cell's top-left corner to the merged card's centre, y.
pub const CELL_CENTRE_Y: f64 = 2.5;

/// Particle spark lifetime, in milliseconds.
pub const SPARK_MILLIS: u64 = 500;
/// Merged tile flash lifetime, in milliseconds.
pub const FLASH_MILLIS: u64 = 180;
/// Newly spawned tile pop lifetime, in milliseconds.
pub const POP_MILLIS: u64 = 120;
/// New-best header pulse lifetime, in milliseconds.
pub const BEST_PULSE_MILLIS: u64 = 600;
/// How many sparks one merge creates.
pub const SPARKS_PER_MERGE: usize = 14;
/// Fixed seed for the effect-only RNG; never the game seed.
pub const EFFECT_RNG_SEED: u64 = 0x2048_5EED_F00D_ABCD;
/// Spark characters, cycled so every merge uses all four.
pub const SPARK_SYMBOLS: [char; 4] = ['·', '✦', '+', '*'];
/// Slowest spark speed in board cells per second.
pub const SPARK_SPEED_MIN: f32 = 5.0;
/// Extra random speed range above [`SPARK_SPEED_MIN`].
pub const SPARK_SPEED_RANGE: f32 = 7.0;

/// One merge ripple: a wave that leaves the merged card and bounces off the walls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ripple {
    /// Merge destination cell, row-major 0..16.
    pub cell: usize,
    /// Value produced by the merge; drives the ripple colour.
    pub value: u32,
    /// Time since the merge.
    pub age: Duration,
}

impl Ripple {
    /// Wavefront radius in play-area cells, with no rounding (SPEC 5.4).
    pub fn radius(&self) -> f64 {
        self.age.as_millis() as f64 * RIPPLE_CELLS_PER_MILLI
    }

    /// Base amplitude before any wall reflection.
    pub fn amplitude(&self) -> f64 {
        1.0 - self.age.as_millis() as f64 / RIPPLE_MILLIS as f64
    }

    /// The merged card's centre in play-area cells.
    pub fn source(&self) -> (f64, f64) {
        let column = (self.cell % 4) as f64;
        let row = (self.cell / 4) as f64;
        (
            column * CELL_PITCH_X + CELL_CENTRE_X,
            row * CELL_PITCH_Y + CELL_CENTRE_Y,
        )
    }

    /// The nine mirrored images of the source as `(x, y, walls crossed)`.
    ///
    /// SPEC 5.4 mirrors about the play-area edges at `-0.5`, `38.5`, `-0.5`
    /// and `22.5`, so the images are `-1 - s` and `77 - s` horizontally and
    /// `-1 - s` and `45 - s` vertically.
    pub fn images(&self) -> [(f64, f64, u32); 9] {
        let (source_x, source_y) = self.source();
        let xs = [(source_x, 0u32), (-1.0 - source_x, 1), (77.0 - source_x, 1)];
        let ys = [(source_y, 0u32), (-1.0 - source_y, 1), (45.0 - source_y, 1)];

        let mut images = [(0.0, 0.0, 0u32); 9];
        let mut index = 0;
        for (x, walls_x) in xs {
            for (y, walls_y) in ys {
                images[index] = (x, y, walls_x + walls_y);
                index += 1;
            }
        }
        images
    }

    /// The strongest amplitude this wave reaches at play-area cell `(x, y)`.
    ///
    /// A cell is on the front for an image when its Chebyshev distance to that
    /// image is within [`RIPPLE_FRONT_WINDOW`] of the radius.
    pub fn amplitude_at(&self, x: i32, y: i32) -> f64 {
        let radius = self.radius();
        let base = self.amplitude();
        let x = f64::from(x);
        let y = f64::from(y);

        self.images()
            .iter()
            .filter_map(|(image_x, image_y, walls)| {
                let distance = (x - image_x).abs().max((y - image_y).abs());
                if (distance - radius).abs() <= RIPPLE_FRONT_WINDOW {
                    Some(base * wall_factor(*walls))
                } else {
                    None
                }
            })
            .fold(0.0, f64::max)
    }

    /// The ripple glyph at play-area cell `(x, y)`, or `None` for a blank cell.
    pub fn glyph_at(&self, x: i32, y: i32) -> Option<char> {
        let amplitude = self.amplitude_at(x, y);
        RIPPLE_THRESHOLDS
            .iter()
            .find(|(_, minimum)| amplitude >= *minimum)
            .map(|(glyph, _)| *glyph)
    }
}

/// `0.55` multiplied once per wall crossed (0, 1 or 2 walls for the nine images).
///
/// The factor is multiplied explicitly instead of using `f64::powi`, whose
/// documented precision is unspecified across platforms and versions.
fn wall_factor(walls: u32) -> f64 {
    match walls {
        0 => 1.0,
        1 => RIPPLE_WALL_FACTOR,
        _ => RIPPLE_WALL_FACTOR * RIPPLE_WALL_FACTOR,
    }
}

/// One drifting spark particle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spark {
    /// Merge destination cell the spark starts from.
    pub origin_cell: usize,
    /// Small starting offset from the cell centre, in board cells.
    pub offset_x: f32,
    /// Small starting offset from the cell centre, in board cells.
    pub offset_y: f32,
    /// Unit direction x.
    pub dir_x: f32,
    /// Unit direction y.
    pub dir_y: f32,
    /// Initial speed in board cells per second.
    pub speed: f32,
    /// One of [`SPARK_SYMBOLS`].
    pub symbol: char,
    /// Time since the merge.
    pub age: Duration,
}

impl Spark {
    /// Distance from the starting point after drag, in board cells.
    pub fn distance(&self) -> f32 {
        let seconds = self.age.as_secs_f32();
        let drag = 1.0 - 0.35 * (seconds / 0.5).min(1.0);
        self.speed * seconds * drag
    }
}

/// One brightened tile, either a merge flash or a spawn pop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flash {
    /// Board cell index.
    pub cell: usize,
    /// How long the flash lives; distinguishes a merge flash from a spawn pop.
    pub lifetime: Duration,
    /// Time since the merge or spawn.
    pub age: Duration,
}

/// All effect state, advanced only by an injected `dt`.
#[derive(Clone, Debug, PartialEq)]
pub struct Effects {
    elapsed: Duration,
    ripples: Vec<Ripple>,
    sparks: Vec<Spark>,
    flashes: Vec<Flash>,
    best_pulse: Option<Duration>,
    rng: Xorshift64,
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

impl Effects {
    /// An empty effect timeline.
    pub fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            ripples: Vec::new(),
            sparks: Vec::new(),
            flashes: Vec::new(),
            best_pulse: None,
            rng: Xorshift64::new(EFFECT_RNG_SEED),
        }
    }

    /// Time since the effects started; drives the plasma phase.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Active ripples, oldest first.
    pub fn ripples(&self) -> &[Ripple] {
        &self.ripples
    }

    /// Active sparks.
    pub fn sparks(&self) -> &[Spark] {
        &self.sparks
    }

    /// Age of the best pulse, or `None` when it is not running.
    pub fn best_pulse_age(&self) -> Option<Duration> {
        self.best_pulse
    }

    /// True while the `BEST` header must pulse.
    pub fn is_best_pulsing(&self) -> bool {
        self.best_pulse.is_some()
    }

    /// True while the merged tile at `cell` must be brightened.
    pub fn is_flashing(&self, cell: usize) -> bool {
        self.flashes.iter().any(|flash| {
            flash.cell == cell && flash.lifetime == Duration::from_millis(FLASH_MILLIS)
        })
    }

    /// True while the spawned tile at `cell` must be brightened.
    pub fn is_popping(&self, cell: usize) -> bool {
        self.flashes
            .iter()
            .any(|flash| flash.cell == cell && flash.lifetime == Duration::from_millis(POP_MILLIS))
    }

    /// Start a ripple, a 180 ms flash and 14 sparks for one merge.
    pub fn trigger_merge(&mut self, merge: Merge) {
        self.ripples.push(Ripple {
            cell: merge.cell,
            value: merge.value,
            age: Duration::ZERO,
        });
        self.flashes.push(Flash {
            cell: merge.cell,
            lifetime: Duration::from_millis(FLASH_MILLIS),
            age: Duration::ZERO,
        });
        self.spawn_sparks(merge.cell);
    }

    /// Start the 120 ms pop for a newly spawned tile.
    pub fn trigger_spawn(&mut self, cell: usize) {
        self.flashes.push(Flash {
            cell,
            lifetime: Duration::from_millis(POP_MILLIS),
            age: Duration::ZERO,
        });
    }

    /// Start the 600 ms `BEST` header pulse.
    pub fn start_best_pulse(&mut self) {
        self.best_pulse = Some(Duration::ZERO);
    }

    /// Drop every running effect; used by restart.
    pub fn clear(&mut self) {
        self.ripples.clear();
        self.sparks.clear();
        self.flashes.clear();
        self.best_pulse = None;
    }

    /// Advance every effect by `dt`.
    ///
    /// Duration `D` means visible while `age < D`, so a flash that reaches its
    /// deadline is removed in the same step.
    pub fn advance(&mut self, dt: Duration) {
        self.elapsed = self.elapsed.saturating_add(dt);

        for ripple in &mut self.ripples {
            ripple.age = ripple.age.saturating_add(dt);
        }
        self.ripples
            .retain(|ripple| ripple.age < Duration::from_millis(RIPPLE_MILLIS));

        for spark in &mut self.sparks {
            spark.age = spark.age.saturating_add(dt);
        }
        self.sparks
            .retain(|spark| spark.age < Duration::from_millis(SPARK_MILLIS));

        for flash in &mut self.flashes {
            flash.age = flash.age.saturating_add(dt);
        }
        self.flashes.retain(|flash| flash.age < flash.lifetime);

        if let Some(age) = self.best_pulse {
            let next = age.saturating_add(dt);
            self.best_pulse = if next < Duration::from_millis(BEST_PULSE_MILLIS) {
                Some(next)
            } else {
                None
            };
        }
    }

    fn spawn_sparks(&mut self, cell: usize) {
        for index in 0..SPARKS_PER_MERGE {
            let angle = (self.rng.next_u64() % 3_600) as f32 / 3_600.0 * TAU;
            let speed = SPARK_SPEED_MIN + (self.rng.next_u64() % 700) as f32 / 100.0;
            let offset_x = (self.rng.next_u64() % 201) as f32 / 100.0 - 1.0;
            let offset_y = (self.rng.next_u64() % 201) as f32 / 100.0 - 1.0;
            self.sparks.push(Spark {
                origin_cell: cell,
                offset_x,
                offset_y,
                dir_x: angle.cos(),
                dir_y: angle.sin(),
                speed,
                symbol: SPARK_SYMBOLS[index % SPARK_SYMBOLS.len()],
                age: Duration::ZERO,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_merge_starts_one_ripple_one_flash_and_fourteen_sparks() {
        let mut effects = Effects::new();
        effects.trigger_merge(Merge { cell: 0, value: 4 });

        assert_eq!(effects.ripples().len(), 1);
        assert_eq!(effects.sparks().len(), SPARKS_PER_MERGE);
        assert!(effects.is_flashing(0));
        assert!(!effects.is_popping(0));
        assert_eq!(effects.ripples()[0].value, 4);
    }

    #[test]
    fn effects_expire_at_their_exact_deadlines() {
        let mut effects = Effects::new();
        effects.trigger_merge(Merge { cell: 0, value: 4 });
        effects.trigger_spawn(9);

        effects.advance(Duration::from_millis(POP_MILLIS - 1));
        assert!(effects.is_popping(9), "pop lasts while age < 120 ms");

        effects.advance(Duration::from_millis(1));
        assert!(!effects.is_popping(9), "pop is gone at 120 ms");
        assert!(effects.is_flashing(0), "flash lasts while age < 180 ms");

        effects.advance(Duration::from_millis(FLASH_MILLIS - POP_MILLIS));
        assert!(!effects.is_flashing(0), "flash is gone at 180 ms");
        assert_eq!(effects.ripples().len(), 1);

        effects.advance(Duration::from_millis(RIPPLE_MILLIS - FLASH_MILLIS - 1));
        assert_eq!(
            effects.ripples().len(),
            1,
            "ripple lasts while age < 900 ms"
        );

        effects.advance(Duration::from_millis(1));
        assert!(effects.ripples().is_empty(), "ripple is gone at 900 ms");
        assert!(
            effects.sparks().is_empty(),
            "sparks are long gone at 900 ms"
        );
    }

    #[test]
    fn ripple_images_mirror_about_the_play_area_walls() {
        let ripple = Ripple {
            cell: 0,
            value: 4,
            age: Duration::ZERO,
        };
        assert_eq!(ripple.source(), (4.5, 2.5));

        let images = ripple.images();
        let mut xs: Vec<f64> = images.iter().map(|(x, _, _)| *x).collect();
        let mut ys: Vec<f64> = images.iter().map(|(_, y, _)| *y).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        xs.dedup();
        ys.dedup();
        assert_eq!(xs, vec![-5.5, 4.5, 72.5]);
        assert_eq!(ys, vec![-3.5, 2.5, 42.5]);

        let walls: Vec<u32> = images.iter().map(|(_, _, walls)| *walls).collect();
        assert_eq!(walls.iter().filter(|walls| **walls == 0).count(), 1);
        assert_eq!(walls.iter().filter(|walls| **walls == 1).count(), 4);
        assert_eq!(walls.iter().filter(|walls| **walls == 2).count(), 4);
    }
}
