use crate::biome::Biome;

pub struct Ocean;

impl Biome for Ocean {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        -5.0
    }
}
