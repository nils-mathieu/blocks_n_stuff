use crate::biome::Biome;

pub struct PineForest;

impl Biome for PineForest {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        25.0
    }
}
