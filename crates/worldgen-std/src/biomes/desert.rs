use crate::biome::Biome;

pub struct Desert;

impl Biome for Desert {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        8.0
    }
}
