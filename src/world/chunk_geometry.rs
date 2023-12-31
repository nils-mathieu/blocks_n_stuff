use std::sync::Arc;

use bns_core::{
    BlockAppearance, BlockFlags, BlockId, BlockVisibility, Chunk, ChunkPos, Face, LocalPos,
    TextureId,
};
use bns_render::data::{QuadFlags, QuadInstance};
use bns_render::{DynamicVertexBuffer, Gpu};

use bitflags::bitflags;
use glam::IVec3;

/// The built geometry of a chunk. This is a wrapper around a vertex buffer that
/// contains the quad instances of the chunk.
pub struct ChunkGeometry {
    /// The quad instances of the chunk.
    ///
    /// When `None`, the vertex buffer has not been created, either because the chunk was never
    /// built, or because it is empty.
    opaque_quads: Option<DynamicVertexBuffer<QuadInstance>>,
    /// The quad instances of the chunk.
    ///
    /// When `None`, the vertex buffer has not been created, either because the chunk was never
    /// built, or because it is empty.
    transparent_quads: Option<DynamicVertexBuffer<QuadInstance>>,
}

impl ChunkGeometry {
    /// Creates a new [`ChunkGeometry`] instance with no built geometry (the chunk is assumed to be
    /// empty.)
    #[inline]
    pub fn new() -> Self {
        Self {
            opaque_quads: None,
            transparent_quads: None,
        }
    }

    /// Returns whether the chunk contains no geometry.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.opaque_quads.is_none() && self.transparent_quads.is_none()
    }

    /// Returns the quad instances of the chunk, if any.
    #[inline]
    pub fn opaque_quad_instances(&self) -> Option<&DynamicVertexBuffer<QuadInstance>> {
        match self.opaque_quads {
            Some(ref buf) if buf.len() > 0 => Some(buf),
            _ => None,
        }
    }

    /// Returns the quad instances of the chunk, if any.
    #[inline]
    pub fn transparent_quad_instances(&self) -> Option<&DynamicVertexBuffer<QuadInstance>> {
        match self.transparent_quads {
            Some(ref buf) if buf.len() > 0 => Some(buf),
            _ => None,
        }
    }
}

/// Contains the state required to upload the geometry of a chunk to the GPU.
pub struct ChunkUploadContext {
    /// The GPU itself.
    gpu: Arc<Gpu>,
    /// A pool of dynamic vertex buffers used to build the geometry of the chunk.
    buffer_pool: Vec<DynamicVertexBuffer<QuadInstance>>,
}

impl ChunkUploadContext {
    /// Creates a new [`ChunkUploadContext`].
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            gpu,
            buffer_pool: Vec::new(),
        }
    }

    /// Acquires a buffer from the pool.
    fn acquire_buffer(&mut self) -> DynamicVertexBuffer<QuadInstance> {
        self.buffer_pool
            .pop()
            .unwrap_or_else(|| DynamicVertexBuffer::new(self.gpu.clone(), 1024))
    }

    /// Releases a buffer to the pool.
    fn release_buffer(&mut self, buf: DynamicVertexBuffer<QuadInstance>) {
        self.buffer_pool.push(buf);
    }

    /// Overwrites the provided [`ChunkGeometry`] instance with the content of the provided
    /// [`ChunkBuildContext`].
    pub fn upload(&mut self, ctx: &ChunkBuildContext, geometry: &mut ChunkGeometry) {
        if !ctx.opaque_quads.is_empty() {
            let quads = geometry
                .opaque_quads
                .get_or_insert_with(|| self.acquire_buffer());
            quads.clear();
            quads.extend(&ctx.opaque_quads);
        } else if let Some(buf) = geometry.opaque_quads.take() {
            self.release_buffer(buf);
        }

        if !ctx.transparent_quads.is_empty() {
            let quads = geometry
                .transparent_quads
                .get_or_insert_with(|| self.acquire_buffer());
            quads.clear();
            quads.extend(&ctx.transparent_quads);
        } else if let Some(buf) = geometry.transparent_quads.take() {
            self.release_buffer(buf);
        }
    }
}

/// Contains some resources useful for building a chunk.
///
/// This mostly includes temporary buffers.
#[derive(Default)]
pub struct ChunkBuildContext {
    opaque_quads: Vec<QuadInstance>,
    transparent_quads: Vec<QuadInstance>,
}

impl ChunkBuildContext {
    /// Clears the current state of the builder, ensuring that the geometry
    /// of the previous chunk is not reused.
    pub fn clear(&mut self) {
        self.opaque_quads.clear();
        self.transparent_quads.clear();
    }

    /// Only build the inner of the chunk.
    ///
    /// This operation does not require lookups to the chunk provider.
    #[profiling::function]
    pub fn build_inner(&mut self, chunk: &Chunk) {
        LocalPos::iter_all().for_each(|pos| build_block(chunk, pos, self));
    }

    /// Only build the outer geometry of the chunk.
    #[profiling::function]
    pub fn build_outer<'a>(&mut self, neighborhood: ChunkNeighborhood<'a>) {
        build_chunk_boundary_x(neighborhood.me, neighborhood.x, self);
        build_chunk_boundary_neg_x(neighborhood.me, neighborhood.neg_x, self);
        build_chunk_boundary_y(neighborhood.me, neighborhood.y, self);
        build_chunk_boundary_neg_y(neighborhood.me, neighborhood.neg_y, self);
        build_chunk_boundary_z(neighborhood.me, neighborhood.z, self);
        build_chunk_boundary_neg_z(neighborhood.me, neighborhood.neg_z, self);
    }
}

/// Contains references to neighboring chunks.
pub struct ChunkNeighborhood<'a> {
    pub me: &'a Chunk,
    pub x: &'a Chunk,
    pub neg_x: &'a Chunk,
    pub y: &'a Chunk,
    pub neg_y: &'a Chunk,
    pub z: &'a Chunk,
    pub neg_z: &'a Chunk,
}

impl<'a> ChunkNeighborhood<'a> {
    /// Returns a [`ChunkNeighborhood`] instance with the provided chunk at the center.
    pub fn from_fn(
        center: ChunkPos,
        mut f: impl FnMut(ChunkPos) -> Option<&'a Chunk>,
    ) -> Option<Self> {
        Some(Self {
            me: f(center)?,
            x: f(center + IVec3::X)?,
            neg_x: f(center + IVec3::NEG_X)?,
            y: f(center + IVec3::Y)?,
            neg_y: f(center + IVec3::NEG_Y)?,
            z: f(center + IVec3::Z)?,
            neg_z: f(center + IVec3::NEG_Z)?,
        })
    }
}

bitflags! {
    /// Voxel faces that are visible.
    #[derive(Debug, Clone, Copy)]
    struct CulledFaces: u8 {
        const X = 1 << 0;
        const NEG_X = 1 << 1;
        const Y = 1 << 2;
        const NEG_Y = 1 << 3;
        const Z = 1 << 4;
        const NEG_Z = 1 << 5;
    }
}

impl CulledFaces {
    /// Returns the [`CulledFaces`] of the block within `chunk` at the provided position.
    ///
    /// If `pos` is at the boundary of the chunk, the faces that are outside of the chunk are
    /// always considered culled.
    pub fn of(chunk: &Chunk, pos: LocalPos) -> Self {
        let me = chunk.get_block(pos);

        let mut result = CulledFaces::all();

        if pos
            .next_x()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::X)
        }
        if pos
            .prev_x()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::NEG_X)
        }
        if pos
            .next_y()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::Y)
        }
        if pos
            .prev_y()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::NEG_Y)
        }
        if pos
            .next_z()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::Z)
        }
        if pos
            .prev_z()
            .is_some_and(|pos| !is_face_culled(me, chunk.get_block(pos)))
        {
            result.remove(CulledFaces::NEG_Z)
        }

        result
    }
}

/// Returns whether a face of `me` against `other` should be culled.
#[inline]
fn is_face_culled(me: BlockId, other: BlockId) -> bool {
    me.info().visibility == BlockVisibility::Invisible
        || other.info().visibility == BlockVisibility::Opaque
        || (me == other && me.info().flags.contains(BlockFlags::CULLS_ITSELF))
}

/// Builds the geometry of one of the inner voxels of the provided chunk.
fn build_block(chunk: &Chunk, pos: LocalPos, ctx: &mut ChunkBuildContext) {
    let culled = CulledFaces::of(chunk, pos);
    let block = chunk.get_block(pos);
    let metadata = chunk.get_appearance(pos);

    let mut base_flags = QuadFlags::from_chunk_index(pos.index());
    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { top, bottom, side } => {
            if !culled.contains(CulledFaces::X) {
                build_regular_face_x(side, chunk, pos, buffer);
            }
            if !culled.contains(CulledFaces::NEG_X) {
                build_regular_face_neg_x(side, chunk, pos, buffer);
            }
            if !culled.contains(CulledFaces::Y) {
                build_regular_face_y(top, chunk, pos, buffer);
            }
            if !culled.contains(CulledFaces::NEG_Y) {
                build_regular_face_neg_y(bottom, chunk, pos, buffer);
            }
            if !culled.contains(CulledFaces::Z) {
                build_regular_face_z(side, chunk, pos, buffer);
            }
            if !culled.contains(CulledFaces::NEG_Z) {
                build_regular_face_neg_z(side, chunk, pos, buffer);
            }
        }
        BlockAppearance::Liquid(surface) => {
            if !culled.contains(CulledFaces::Y) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::Y | QuadFlags::OFFSET_1 | QuadFlags::LIQUID,
                    texture: surface as u32,
                });
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::NEG_Y | QuadFlags::OFFSET_7 | QuadFlags::LIQUID,
                    texture: surface as u32,
                });
            }
        }
        BlockAppearance::Flat(texture) => {
            // SAFETY:
            //  The block apperance is `Flat`.
            let face = unsafe { metadata.flat };
            base_flags |= QuadFlags::OVERLAY;

            match face {
                Face::X if !culled.contains(CulledFaces::X) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::X,
                        texture: texture as u32,
                    });
                }
                Face::NegX if !culled.contains(CulledFaces::NEG_X) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::NEG_X,
                        texture: texture as u32,
                    });
                }
                Face::Y if !culled.contains(CulledFaces::Y) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::Y,
                        texture: texture as u32,
                    });
                }
                Face::NegY if !culled.contains(CulledFaces::NEG_Y) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::NEG_Y,
                        texture: texture as u32,
                    });
                }
                Face::Z if !culled.contains(CulledFaces::Z) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::Z,
                        texture: texture as u32,
                    });
                }
                Face::NegZ if !culled.contains(CulledFaces::NEG_Z) => {
                    buffer.push(QuadInstance {
                        flags: base_flags | QuadFlags::NEG_Z,
                        texture: texture as u32,
                    });
                }
                _ => (),
            }
        }
    }
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_x(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(Chunk::SIDE - 1, a, b),
                LocalPos::from_xyz_unchecked(0, a, b),
            )
        },
        |pos| build_single_face_x(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_neg_x(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(0, a, b),
                LocalPos::from_xyz_unchecked(Chunk::SIDE - 1, a, b),
            )
        },
        |pos| build_single_face_neg_x(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_y(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(a, Chunk::SIDE - 1, b),
                LocalPos::from_xyz_unchecked(a, 0, b),
            )
        },
        |pos| build_single_face_y(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_neg_y(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(a, 0, b),
                LocalPos::from_xyz_unchecked(a, Chunk::SIDE - 1, b),
            )
        },
        |pos| build_single_face_neg_y(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_z(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(a, b, Chunk::SIDE - 1),
                LocalPos::from_xyz_unchecked(a, b, 0),
            )
        },
        |pos| build_single_face_z(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
fn build_chunk_boundary_neg_z(data: &Chunk, other: &Chunk, ctx: &mut ChunkBuildContext) {
    build_chunk_boundary(
        data,
        other,
        |a, b| unsafe {
            (
                LocalPos::from_xyz_unchecked(a, b, 0),
                LocalPos::from_xyz_unchecked(a, b, Chunk::SIDE - 1),
            )
        },
        |pos| build_single_face_neg_z(pos, data, ctx),
    );
}

/// Builds the boundary of the provided chunk based on its content and the content of the
/// adjacent chunk.
///
/// The `coords` function is used to convert the coordinates of the chunk's face into a local
/// position in the chunk/adjacent chunk.
fn build_chunk_boundary(
    data: &Chunk,
    other: &Chunk,
    mut coords: impl FnMut(i32, i32) -> (LocalPos, LocalPos),
    mut build: impl FnMut(LocalPos),
) {
    for a in 0..Chunk::SIDE {
        for b in 0..Chunk::SIDE {
            let (pos, other_pos) = coords(a, b);
            if !is_face_culled(data.get_block(pos), other.get_block(other_pos)) {
                build(pos)
            }
        }
    }
}

/// Builds a single face of a block.
fn build_single_face_x(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { side, .. } => {
            build_regular_face_x(side, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Builds a single face of a block.
fn build_single_face_neg_x(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { side, .. } => {
            build_regular_face_neg_x(side, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Builds a single face of a block.
fn build_single_face_z(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { side, .. } => {
            build_regular_face_z(side, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Builds a single face of a block.
fn build_single_face_neg_z(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { side, .. } => {
            build_regular_face_neg_z(side, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Builds a single face of a block.
fn build_single_face_y(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);
    let metadata = chunk.get_appearance(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { top, .. } => {
            build_regular_face_y(top, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(surface) => {
            buffer.push(QuadInstance {
                flags: QuadFlags::from_chunk_index(pos.index())
                    | QuadFlags::OFFSET_1
                    | QuadFlags::Y
                    | QuadFlags::LIQUID,
                texture: surface as u32,
            });
            buffer.push(QuadInstance {
                flags: QuadFlags::from_chunk_index(pos.index())
                    | QuadFlags::NEG_Y
                    | QuadFlags::OFFSET_7
                    | QuadFlags::LIQUID,
                texture: surface as u32,
            });
        }
        BlockAppearance::Flat(texture) => {
            // SAFETY:
            //  The block apperance is `Flat`.
            let face = unsafe { metadata.flat };

            if face == Face::Y {
                buffer.push(QuadInstance {
                    flags: QuadFlags::from_chunk_index(pos.index())
                        | QuadFlags::OVERLAY
                        | QuadFlags::Y,
                    texture: texture as u32,
                });
            }
        }
    }
}

/// Builds a single face of a block.
fn build_single_face_neg_y(pos: LocalPos, chunk: &Chunk, ctx: &mut ChunkBuildContext) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { bottom, .. } => {
            build_regular_face_neg_y(bottom, chunk, pos, buffer);
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Computes the ambient occlusion flags for a face facing the positive X axis.
fn compute_ambient_occlusion_x(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.prev_z(),
        pos.next_z(),
        pos.prev_y(),
        pos.next_y(),
    )
}

/// Computes the ambient occlusion flags for a face facing the negative X axis.
fn compute_ambient_occlusion_neg_x(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.next_z(),
        pos.prev_z(),
        pos.prev_y(),
        pos.next_y(),
    )
}

/// Computes the ambient occlusion flags for a face facing the positive Y axis.
fn compute_ambient_occlusion_y(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.prev_x(),
        pos.next_x(),
        pos.prev_z(),
        pos.next_z(),
    )
}

/// Computes the ambient occlusion flags for a face facing the negative Y axis.
fn compute_ambient_occlusion_neg_y(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.prev_x(),
        pos.next_x(),
        pos.next_z(),
        pos.prev_z(),
    )
}

/// Computes the ambient occlusion flags for a face facing the positive Z axis.
fn compute_ambient_occlusion_z(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.next_x(),
        pos.prev_x(),
        pos.prev_y(),
        pos.next_y(),
    )
}

/// Computes the ambient occlusion flags for a face facing the negative Z axis.
fn compute_ambient_occlusion_neg_z(chunk: &Chunk, pos: LocalPos) -> QuadFlags {
    compute_ambient_occlusion(
        chunk,
        pos.prev_x(),
        pos.next_x(),
        pos.prev_y(),
        pos.next_y(),
    )
}

/// Computes the ambient occlusion of a block face.
#[inline]
fn compute_ambient_occlusion(
    chunk: &Chunk,
    left: Option<LocalPos>,
    right: Option<LocalPos>,
    bottom: Option<LocalPos>,
    top: Option<LocalPos>,
) -> QuadFlags {
    let mut flags = QuadFlags::empty();

    if left.is_some_and(|pos| chunk.get_block(pos).info().visibility == BlockVisibility::Opaque) {
        flags |= QuadFlags::OCCLUDED_LEFT
    }

    if right.is_some_and(|pos| chunk.get_block(pos).info().visibility == BlockVisibility::Opaque) {
        flags |= QuadFlags::OCCLUDED_RIGHT
    }

    if bottom.is_some_and(|pos| chunk.get_block(pos).info().visibility == BlockVisibility::Opaque) {
        flags |= QuadFlags::OCCLUDED_BOTTOM
    }

    if top.is_some_and(|pos| chunk.get_block(pos).info().visibility == BlockVisibility::Opaque) {
        flags |= QuadFlags::OCCLUDED_TOP
    }

    flags
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_x(tex: TextureId, chunk: &Chunk, pos: LocalPos, out: &mut Vec<QuadInstance>) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::X;

    if let Some(pos) = pos.next_x() {
        flags |= compute_ambient_occlusion_x(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_neg_x(
    tex: TextureId,
    chunk: &Chunk,
    pos: LocalPos,
    out: &mut Vec<QuadInstance>,
) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::NEG_X;

    if let Some(pos) = pos.prev_x() {
        flags |= compute_ambient_occlusion_neg_x(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_z(tex: TextureId, chunk: &Chunk, pos: LocalPos, out: &mut Vec<QuadInstance>) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::Z;

    if let Some(pos) = pos.next_z() {
        flags |= compute_ambient_occlusion_z(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_neg_z(
    tex: TextureId,
    chunk: &Chunk,
    pos: LocalPos,
    out: &mut Vec<QuadInstance>,
) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::NEG_Z;

    if let Some(pos) = pos.prev_z() {
        flags |= compute_ambient_occlusion_neg_z(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_y(tex: TextureId, chunk: &Chunk, pos: LocalPos, out: &mut Vec<QuadInstance>) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::Y;

    if let Some(pos) = pos.next_y() {
        flags |= compute_ambient_occlusion_y(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}

/// Builds a face that has the "Regular" appearance.
fn build_regular_face_neg_y(
    tex: TextureId,
    chunk: &Chunk,
    pos: LocalPos,
    out: &mut Vec<QuadInstance>,
) {
    let mut flags = QuadFlags::from_chunk_index(pos.index()) | QuadFlags::NEG_Y;

    if let Some(pos) = pos.prev_y() {
        flags |= compute_ambient_occlusion_neg_y(chunk, pos);
    }

    out.push(QuadInstance {
        flags,
        texture: tex as u32,
    });
}
