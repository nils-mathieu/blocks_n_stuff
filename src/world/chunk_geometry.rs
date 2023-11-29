use std::sync::Arc;

use bns_core::{BlockAppearance, Chunk, LocalPos};

use crate::gfx::render_data::QuadInstance;
use crate::gfx::Gpu;

/// Contains some resources useful for building a chunk.
///
/// This mostly includes temporary buffers.
pub struct ChunkBuildContext {
    gpu: Arc<Gpu>,
    quads: Vec<QuadInstance>,
}

impl ChunkBuildContext {
    /// Creates a new [`ChunkBuildContext`].
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            gpu,
            quads: Vec::new(),
        }
    }

    /// Resets the context.
    fn reset(&mut self) {
        self.quads.clear();
    }
}

/// The built geometry of a chunk. This is a wrapper around a vertex buffer that
/// contains the quad instances of the chunk.
pub struct ChunkGeometry {
    /// The quad instances of the chunk.
    ///
    /// When `None`, the vertex buffer has not been created.
    pub quads: Option<(u32, wgpu::Buffer)>,
}

impl ChunkGeometry {
    /// Creates a new [`ChunkGeometry`] instance.
    pub fn new() -> Self {
        Self { quads: None }
    }

    /// Builds the geometry of a chunk.
    pub fn build(&mut self, neighborhood: ChunkNeighborhood, context: &mut ChunkBuildContext) {
        context.reset();

        // TODO: actually perform some culling.
        for local_pos in LocalPos::iter_all() {
            match neighborhood.this.get_block(local_pos).info().appearance {
                BlockAppearance::Invisible => (),
                BlockAppearance::Regular { top, bottom, side } => {
                    context.quads.extend_from_slice(&[
                        QuadInstance::from_texture(side)
                            | QuadInstance::X
                            | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::from_texture(side)
                            | QuadInstance::NEG_X
                            | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::from_texture(top)
                            | QuadInstance::Y
                            | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::from_texture(bottom)
                            | QuadInstance::NEG_Y
                            | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::from_texture(side)
                            | QuadInstance::Z
                            | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::from_texture(side)
                            | QuadInstance::NEG_Z
                            | QuadInstance::from_local_pos(local_pos),
                    ]);
                }
            }
        }

        if context.quads.is_empty() {
            self.quads = None;
        } else {
            let (count, buf) = self.quads.get_or_insert_with(|| {
                (
                    0,
                    create_quad_vertex_buffer(&context.gpu, context.quads.len() as _),
                )
            });

            *count = context.quads.len() as _;
            context
                .gpu
                .queue
                .write_buffer(buf, 0, bytemuck::cast_slice(&context.quads));
        }
    }
}

/// Creates the vertex buffer that contains the quad instances of a chunk.
fn create_quad_vertex_buffer(gpu: &Gpu, capacity: wgpu::BufferAddress) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunk quad instances"),
        size: capacity * std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The neighborhood of a chunk.
pub struct ChunkNeighborhood<'a> {
    /// The data of the chunk that's being built.
    pub this: &'a Chunk,
    /// The chunk that's on the positive X side of this chunk.
    pub x: &'a Chunk,
    /// The chunk that's on the negative X side of this chunk.
    pub nx: &'a Chunk,
    /// The chunk that's on the positive Y side of this chunk.
    pub y: &'a Chunk,
    /// The chunk that's on the negative Y side of this chunk.
    pub ny: &'a Chunk,
    /// The chunk that's on the positive Z side of this chunk.
    pub z: &'a Chunk,
    /// The chunk that's on the negative Z side of this chunk.
    pub nz: &'a Chunk,
}
