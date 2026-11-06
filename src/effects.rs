//! Deterministic CPU-rendered effects: shockwaves, sparks, flashes and the best pulse.
//!
//! The module is pure: time arrives through [`Effects::advance`] as an injected
//! `dt`, and the only randomness is a private effect RNG. The game rules and the
//! game RNG are never touched here.

use core::f32::consts::TAU;
use core::time::Duration;

use crate::game::Merge;
use crate::rng::Xorshift64;

/// Merge shockwave lifetime, in milliseconds.
pub const SHOCKWAVE_MILLIS: u64 = 450;
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
/// Shockwave ring characters, from the inner band to the outer band.
pub const RING_SYMBOLS: [char; 3] = ['▓', '▒', '░'];
/// Slowest spark speed in board cells per second.
pub const SPARK_SPEED_MIN: f32 = 5.0;
/// Extra random speed range above [`SPARK_SPEED_MIN`].
pub const SPARK_SPEED_RANGE: f32 = 7.0;

/// One expanding merge ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shockwave {
    /// Merge destination cell, row-major 0..16.
    pub cell: usize,
    /// Value produced by the merge; drives the ring colour.
    pub value: u32,
    /// Time since the merge.
    pub age: Duration,
}

impl Shockwave {
    /// Ring radius in terminal cells, derived only from the effect age.
    pub fn radius(&self) -> i32 {
        1 + (self.age.as_millis() as i32 * 3) / 100
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
    shockwaves: Vec<Shockwave>,
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
            shockwaves: Vec::new(),
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

    /// Active shockwaves, oldest first.
    pub fn shockwaves(&self) -> &[Shockwave] {
        &self.shockwaves
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

    /// Start a shockwave, a 180 ms flash and 14 sparks for one merge.
    pub fn trigger_merge(&mut self, merge: Merge) {
        self.shockwaves.push(Shockwave {
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
        self.shockwaves.clear();
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

        for wave in &mut self.shockwaves {
            wave.age = wave.age.saturating_add(dt);
        }
        self.shockwaves
            .retain(|wave| wave.age < Duration::from_millis(SHOCKWAVE_MILLIS));

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
    fn a_merge_starts_one_wave_one_flash_and_fourteen_sparks() {
        let mut effects = Effects::new();
        effects.trigger_merge(Merge { cell: 0, value: 4 });

        assert_eq!(effects.shockwaves().len(), 1);
        assert_eq!(effects.sparks().len(), SPARKS_PER_MERGE);
        assert!(effects.is_flashing(0));
        assert!(!effects.is_popping(0));
        assert_eq!(effects.shockwaves()[0].value, 4);
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
        assert_eq!(effects.shockwaves().len(), 1);

        effects.advance(Duration::from_millis(SHOCKWAVE_MILLIS - FLASH_MILLIS - 1));
        assert_eq!(
            effects.shockwaves().len(),
            1,
            "wave lasts while age < 450 ms"
        );

        effects.advance(Duration::from_millis(1));
        assert!(effects.shockwaves().is_empty(), "wave is gone at 450 ms");
        assert_eq!(effects.sparks().len(), SPARKS_PER_MERGE);

        effects.advance(Duration::from_millis(SPARK_MILLIS - SHOCKWAVE_MILLIS));
        assert!(effects.sparks().is_empty(), "sparks are gone at 500 ms");
    }
}
