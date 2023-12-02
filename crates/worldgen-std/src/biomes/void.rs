use crate::biome::Biome;

pub struct Void;

impl Biome for Void {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        0.0
    }
}
