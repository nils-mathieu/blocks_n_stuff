use bns_core::{Chunk, ChunkPos};
use glam::IVec3;

use crate::biome::Biome;
use crate::column_gen::ColumnGen;
use crate::GenCtx;

pub struct Void;

impl Biome for Void {
    fn height(&self, _pos: glam::IVec2) -> f32 {
        0.0
    }

    fn geological_stage(&self, pos: ChunkPos, column: &ColumnGen, ctx: &GenCtx, chunk: &mut Chunk) {
        let _ = (pos, column, ctx, chunk);
    }

    fn debug_info(&self, buf: &mut String, pos: IVec3) {
        let _ = (buf, pos);
    }
}
