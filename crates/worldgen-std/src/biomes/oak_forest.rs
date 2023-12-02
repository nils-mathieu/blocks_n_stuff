use crate::biome::Biome;

pub struct OakForest;

impl Biome for OakForest {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        14.0
    }
}
