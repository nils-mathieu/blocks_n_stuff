use std::sync::Arc;

use crate::gfx::render_data::QuadInstance;
use crate::gfx::Gpu;

use super::{BlockAppearance, ChunkData, LocalPos, BLOCK_REGISTRY};

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
            match BLOCK_REGISTRY[neighborhood.this[local_pos]].appearance {
                BlockAppearance::Invisible => (),
                BlockAppearance::Regular { .. } => {
                    context.quads.extend_from_slice(&[
                        QuadInstance::X | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::NEG_X | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::Y | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::NEG_Y | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::Z | QuadInstance::from_local_pos(local_pos),
                        QuadInstance::NEG_Z | QuadInstance::from_local_pos(local_pos),
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
    pub this: &'a ChunkData,
    /// The chunk that's on the positive X side of this chunk.
    pub x: &'a ChunkData,
    /// The chunk that's on the negative X side of this chunk.
    pub nx: &'a ChunkData,
    /// The chunk that's on the positive Y side of this chunk.
    pub y: &'a ChunkData,
    /// The chunk that's on the negative Y side of this chunk.
    pub ny: &'a ChunkData,
    /// The chunk that's on the positive Z side of this chunk.
    pub z: &'a ChunkData,
    /// The chunk that's on the negative Z side of this chunk.
    pub nz: &'a ChunkData,
}
