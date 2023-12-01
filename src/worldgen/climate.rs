use bns_rng::noises::Simplex2;
use bns_rng::{FromRng, Noise};

use bytemuck::Zeroable;
use glam::IVec2;

/// Stores the climate of a particular column of the world.
#[derive(Debug, Clone, Zeroable)]
pub struct Climate {
    /// The temperature value for this column.
    ///
    /// Ranges from `-1` to `1`.
    pub temperature: f32,
    /// The humidity value for this column.
    ///
    /// Ranges from `-1` to `1`.
    pub humidity: f32,
    /// The continentalness value for this column.
    ///
    /// Ranges from `-1` to `1`.
    pub continentalness: f32,
    /// The height of the block.
    pub height: i32,
}

/// COntains the state required to compute climate information about a column (see [`Climate`]).
#[derive(FromRng)]
pub struct ClimateGenerator {
    continentalness: [Simplex2; 6],
    humidity: [Simplex2; 6],
    temperature: [Simplex2; 6],
}

impl ClimateGenerator {
    /// Returns the [`Climate`] associated with the provided position.
    pub fn sample_climate(&self, pos: IVec2) -> Climate {
        let c = sample_base_continentalness(pos, &self.continentalness);
        let mut h = sample_base_humidity(pos, &self.humidity);
        let mut t = sample_base_temperature(pos, &self.temperature);

        // Zones that have high continentalness tend to be drier.
        h -= c * 0.25;

        // Zones that have high continentalness tend to be hotter/colder.
        if t > 0.0 {
            t += c * 0.25;
        } else {
            t -= c * 0.25;
        }

        // Zones that have high humidity tend to be hotter.
        if h > 0.0 {
            t += h * 0.25;
        }

        // Generate the hight of the block based on the previous continentalness value.
        let height = compute_height(c);

        Climate {
            continentalness: c,
            humidity: h,
            temperature: t,
            height,
        }
    }
}

/// Samples the base temperature at the given position.
///
/// # Remarks
///
/// This does not return the final temperature of the block, only a base value that will be
/// affected by other factors.
fn sample_base_temperature(pos: IVec2, noises: &[Simplex2; 6]) -> f32 {
    let x = pos.x as f32 / 1800.0;
    let y = pos.y as f32 / 1800.0;

    let mut ret = 0.0;
    ret += noises[0].sample([x, y]);
    ret += noises[1].sample([x * 2.0, y * 2.0]) * 0.5;
    ret += noises[2].sample([x * 4.0, y * 4.0]) * 0.25;
    ret += noises[3].sample([x * 8.0, y * 8.0]) * 0.125;
    ret += noises[4].sample([x * 16.0, y * 16.0]) * 0.0625;
    ret += noises[5].sample([x * 32.0, y * 32.0]) * 0.03125;
    ret
}

/// Samples the base humidity at the given position.
///
/// # Remarks
///
/// This does not return the final humidity of the block, only a base value that will be
/// affected by other factors.
fn sample_base_humidity(pos: IVec2, noises: &[Simplex2; 6]) -> f32 {
    let x = pos.x as f32 / 1800.0;
    let y = pos.y as f32 / 1800.0;

    let mut ret = 0.0;
    ret += noises[0].sample([x, y]);
    ret += noises[1].sample([x * 2.0, y * 2.0]) * 0.5;
    ret += noises[2].sample([x * 4.0, y * 4.0]) * 0.25;
    ret += noises[3].sample([x * 8.0, y * 8.0]) * 0.125;
    ret += noises[4].sample([x * 16.0, y * 16.0]) * 0.0625;
    ret += noises[5].sample([x * 32.0, y * 32.0]) * 0.03125;
    ret
}

/// Samples the base continentalness at the given position.
///
/// # Remarks
///
/// This does not return the final continentalness of the block, only a base value that will be
/// affected by other factors.
fn sample_base_continentalness(pos: IVec2, noises: &[Simplex2; 6]) -> f32 {
    let x = pos.x as f32 / 1500.0;
    let y = pos.y as f32 / 1500.0;

    let mut ret = 0.0;
    ret += noises[0].sample([x, y]);
    ret += noises[1].sample([x * 2.0, y * 2.0]) * 0.5;
    ret += noises[2].sample([x * 4.0, y * 4.0]) * 0.25;
    ret += noises[3].sample([x * 8.0, y * 8.0]) * 0.125;
    ret += noises[4].sample([x * 16.0, y * 16.0]) * 0.0625;
    ret += noises[5].sample([x * 32.0, y * 32.0]) * 0.03125;
    ret
}

/// Computes the base height value of a block given its continentalness.
fn compute_height(c: f32) -> i32 {
    if c < -0.5 {
        // Deep ocean.
        remap(c, -1.0, -0.5, -64.0, -16.0) as i32
    } else if c < 0.0 {
        // Shallow ocean.
        remap(c, -0.5, 0.0, -16.0, 0.0) as i32
    } else if c < 0.25 {
        // Coast
        remap(c, 0.0, 0.25, 0.0, 16.0) as i32
    } else if c < 0.75 {
        // Plains
        remap(c, 0.25, 0.75, 16.0, 32.0) as i32
    } else {
        // Mountains
        remap(c, 0.75, 1.0, 32.0, 64.0) as i32
    }
}

/// Remaps the provided value from the old range to the new range.
#[inline]
fn remap(x: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    (x - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}
