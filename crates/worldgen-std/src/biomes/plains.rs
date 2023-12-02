use crate::biome::Biome;

pub struct Plains;

impl Biome for Plains {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        8.0
    }
}
