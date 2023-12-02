use bns_rng::noises::{Mixer, Simplex2, Voronoi2};
use bns_rng::{FromRng, Noise};

use glam::IVec2;
use smallvec::SmallVec;

use crate::biome::BiomeId;

/// Represents the climate of a particular tile.
///
/// This is used to determine the biome of a tile.
pub struct Climate {
    /// The continentality of the tile.
    ///
    /// The more continental a tile is, the more likely it is to be on land. The less it is, the
    /// more likely it is to be in an ocean.
    pub continentality: f32,

    /// The temperature of the tile.
    ///
    /// The hotter a tile is, the more likely it is to generate a hot biome. The colder it is, the
    /// more likely it is to generate a cold biome.
    pub temperature: f32,

    /// The humidity of the tile.
    ///
    /// The more humid a tile is, the more likely it is to generate a wet biome. The less humid it
    /// is, the more likely it is to generate a dry biome.
    pub humidity: f32,
}

/// The map used to determine the climate of a tile.
#[derive(FromRng, Clone, Debug)]
pub struct ClimateMap {
    continentality: Simplex2,
    temperature: Simplex2,
    humidity: Simplex2,
}

impl ClimateMap {
    /// The overall scale of the climate map.
    ///
    /// # Notes
    ///
    /// This scale is applied multiplicatively to the scale of the [`BiomeCellMap`], meaning that
    /// the overall scale of the biome map is `ClimateMap::SCALE * BiomeCellMap::SCALE`.
    pub const SCALE: f32 = 1.0 / 8.0;

    /// The individual scale of the continentality map.
    pub const CONTINENTALITY_SCALE: f32 = 1.0;

    /// The individual scale of the temperature map.
    pub const TEMPERATURE_SCALE: f32 = 2.0;

    /// The individual scale of the humidity map.
    pub const HUMIDITY_SCALE: f32 = 2.0;
}

impl Noise<BiomeCell> for ClimateMap {
    type Output = Climate;

    fn sample(&self, input: BiomeCell) -> Self::Output {
        let pos = input.as_vec2() * Self::SCALE;

        let continentality = self
            .continentality
            .sample((pos * Self::CONTINENTALITY_SCALE).into());
        let temperature = self
            .temperature
            .sample((pos * Self::TEMPERATURE_SCALE).into());
        let humidity = self.humidity.sample((pos * Self::HUMIDITY_SCALE).into());

        Climate {
            continentality,
            temperature,
            humidity,
        }
    }
}

/// A particular biome cell.
pub type BiomeCell = IVec2;

#[derive(FromRng, Clone, Debug)]
pub struct BiomeCellMap {
    base_noise: Voronoi2,
    displacement_x: Simplex2,
    displacement_y: Simplex2,
}

impl BiomeCellMap {
    /// The scale of the biome map.
    pub const SCALE: f32 = 1.0 / 24.0;

    /// The roughness of the biome map.
    ///
    /// This is linked to the displacement. The higher the roughness, the more jagged the borders
    /// will be.
    pub const ROUGHNESS: f32 = 2.0;
    /// The displacement of the biome map.
    ///
    /// This is linked to the roughness. The higher the displacement, the more amplitude the
    /// jagged edges will have.
    pub const DISPLACEMENT: f32 = 1.0 / 8.0;
}

impl Noise<IVec2> for BiomeCellMap {
    type Output = BiomeCell;

    fn sample(&self, pos: IVec2) -> Self::Output {
        let pos = pos.as_vec2() * Self::SCALE * 0.5;
        let disp_x =
            self.displacement_x.sample((pos * Self::ROUGHNESS).into()) * Self::DISPLACEMENT;
        let disp_y =
            self.displacement_y.sample((pos * Self::ROUGHNESS).into()) * Self::DISPLACEMENT;
        self.base_noise
            .sample([pos.x + disp_x, pos.y + disp_y])
            .into()
    }
}

/// This type contains the state required to generate the biome map.
#[derive(FromRng, Clone, Debug)]
pub struct BiomeMap {
    climate: ClimateMap,
    cells: BiomeCellMap,
    hasher: Mixer<2>,
}

impl Noise<IVec2> for BiomeMap {
    type Output = BiomeId;

    fn sample(&self, pos: IVec2) -> Self::Output {
        let cell = self.cells.sample(pos);
        let climate = self.climate.sample(cell);
        let biomes: SmallVec<[BiomeId; 8]> = BiomeId::iter_for_climate(&climate).collect();
        let total_weight = biomes.iter().map(|id| id.info().weight).sum::<u32>();
        let mut biome_value =
            self.hasher.sample([cell.x as u64, cell.y as u64]) as u32 % total_weight;

        let mut iter = biomes.iter();
        while biome_value > 0 {
            // SAFETY:
            //  The biome value is non-zero, meaning that some biome must have existed to
            //  increase the total weight.
            let biome = unsafe { iter.next().unwrap_unchecked() };

            if biome_value < biome.info().weight {
                return *biome;
            }

            biome_value -= biome.info().weight;
        }

        BiomeId::Void
    }
}
