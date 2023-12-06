use std::sync::Arc;

use bitflags::bitflags;
use bns_core::{BlockAppearance, BlockFlags, BlockId, BlockVisibility, Chunk, Face, LocalPos};
use bns_render::data::{QuadFlags, QuadInstance};
use bns_render::{DynamicVertexBuffer, Gpu};
use bytemuck::NoUninit;

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

/// Contains some resources useful for building a chunk.
///
/// This mostly includes temporary buffers.
pub struct ChunkBuildContext {
    gpu: Arc<Gpu>,
    opaque_quads: Vec<QuadInstance>,
    transparent_quads: Vec<QuadInstance>,
}

impl ChunkBuildContext {
    /// Creates a new [`ChunkBuildContext`].
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            gpu,
            opaque_quads: Vec::new(),
            transparent_quads: Vec::new(),
        }
    }

    /// Resets the context for a new chunk.
    ///
    /// This allows to reuse the same context for multiple chunks to save on allocations.
    #[inline]
    pub fn reset(&mut self) {
        self.opaque_quads.clear();
        self.transparent_quads.clear();
    }

    /// Builds the inner geometry of the provided chunk based on its content.
    ///
    /// Note that the neighboring chunks are *not* taken into account for culling, and the outer
    /// faces of the chunk are never built.
    #[profiling::function]
    pub fn build_inner(&mut self, data: &Chunk) {
        for local_pos in LocalPos::iter_all() {
            build_block(data, local_pos, self);
        }
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the positive X axis).
    #[profiling::function]
    pub fn build_boundary_x(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(Chunk::SIDE - 1, a, b),
                    LocalPos::from_xyz_unchecked(0, a, b),
                )
            },
            |pos| {
                build_single_face_side(
                    pos,
                    data,
                    QuadFlags::from_chunk_index(pos.index()) | QuadFlags::X,
                    self,
                )
            },
        )
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the negative X axis).
    #[profiling::function]
    pub fn build_boundary_neg_x(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(0, a, b),
                    LocalPos::from_xyz_unchecked(Chunk::SIDE - 1, a, b),
                )
            },
            |pos| {
                build_single_face_side(
                    pos,
                    data,
                    QuadFlags::from_chunk_index(pos.index()) | QuadFlags::NEG_X,
                    self,
                )
            },
        )
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the positive Y axis).
    #[profiling::function]
    pub fn build_boundary_y(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(a, Chunk::SIDE - 1, b),
                    LocalPos::from_xyz_unchecked(a, 0, b),
                )
            },
            |pos| build_single_face_top(pos, data, QuadFlags::from_chunk_index(pos.index()), self),
        )
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the negative Y axis).
    #[profiling::function]
    pub fn build_boundary_neg_y(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(a, 0, b),
                    LocalPos::from_xyz_unchecked(a, Chunk::SIDE - 1, b),
                )
            },
            |pos| {
                build_single_face_bottom(pos, data, QuadFlags::from_chunk_index(pos.index()), self)
            },
        )
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the positive Z axis).
    #[profiling::function]
    pub fn build_boundary_z(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(a, b, Chunk::SIDE - 1),
                    LocalPos::from_xyz_unchecked(a, b, 0),
                )
            },
            |pos| {
                build_single_face_side(
                    pos,
                    data,
                    QuadFlags::from_chunk_index(pos.index()) | QuadFlags::Z,
                    self,
                )
            },
        )
    }

    /// Builds the boundary of the provided chunk based on its content and the content of the
    /// adjacent chunk (on the negative Z axis).
    #[profiling::function]
    pub fn build_boundary_neg_z(&mut self, data: &Chunk, other: &Chunk) {
        build_chunk_boundary(
            data,
            other,
            |a, b| unsafe {
                (
                    LocalPos::from_xyz_unchecked(a, b, 0),
                    LocalPos::from_xyz_unchecked(a, b, Chunk::SIDE - 1),
                )
            },
            |pos| {
                build_single_face_side(
                    pos,
                    data,
                    QuadFlags::from_chunk_index(pos.index()) | QuadFlags::NEG_Z,
                    self,
                )
            },
        )
    }

    /// Applies this [`ChunkBuildContext`] to the provided [`ChunkGeometry`] instance.
    ///
    /// The old geometry of the chunk is kept!
    pub fn append_to(&self, geometry: &mut ChunkGeometry) {
        fn extend_or_create<T>(gpu: &Arc<Gpu>, buf: &mut Option<DynamicVertexBuffer<T>>, data: &[T])
        where
            T: NoUninit,
        {
            match buf {
                Some(buf) => buf.extend(data),
                None => *buf = Some(DynamicVertexBuffer::new_with_data(gpu.clone(), data)),
            }
        }

        if !self.opaque_quads.is_empty() {
            extend_or_create(&self.gpu, &mut geometry.opaque_quads, &self.opaque_quads);
        }
        if !self.transparent_quads.is_empty() {
            extend_or_create(
                &self.gpu,
                &mut geometry.transparent_quads,
                &self.transparent_quads,
            );
        }
    }

    /// Overwrites the provided [`ChunkGeometry`] instance with this [`ChunkBuildContext`].
    pub fn overwrite_to(&mut self, geometry: &mut ChunkGeometry) {
        if let Some(buf) = &mut geometry.opaque_quads {
            buf.clear();
        }
        if let Some(buf) = &mut geometry.transparent_quads {
            buf.clear();
        }
        self.append_to(geometry);
    }
}

bitflags! {
    /// Voxel faces that have been culled.
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
    pub fn of(chunk: &Chunk, pos: LocalPos) -> Self {
        const MAX: i32 = Chunk::SIDE - 1;
        const MIN: i32 = 0;

        unsafe {
            let me = chunk.get_block(pos);

            let mut result = CulledFaces::empty();

            if pos.x() == MAX || is_face_culled(me, chunk.get_block(pos.add_x_unchecked(1))) {
                result |= CulledFaces::X;
            }
            if pos.x() == MIN || is_face_culled(me, chunk.get_block(pos.add_x_unchecked(-1))) {
                result |= CulledFaces::NEG_X;
            }
            if pos.y() == MAX || is_face_culled(me, chunk.get_block(pos.add_y_unchecked(1))) {
                result |= CulledFaces::Y;
            }
            if pos.y() == MIN || is_face_culled(me, chunk.get_block(pos.add_y_unchecked(-1))) {
                result |= CulledFaces::NEG_Y;
            }
            if pos.z() == MAX || is_face_culled(me, chunk.get_block(pos.add_z_unchecked(1))) {
                result |= CulledFaces::Z;
            }
            if pos.z() == MIN || is_face_culled(me, chunk.get_block(pos.add_z_unchecked(-1))) {
                result |= CulledFaces::NEG_Z;
            }

            result
        }
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
///
/// # Remarks
///
/// This function takes [`IntoCoord`] implementations as an input so that it can be monomorphized
/// into a version that does not perform bound checks at the chunk's boundaries. In the case
/// where the coordinates are known to be within the chunk's boundaries,
/// [`Coord`](coord::Coord) can be used as an input.
fn build_block(chunk: &Chunk, pos: LocalPos, ctx: &mut ChunkBuildContext) {
    let culled = CulledFaces::of(chunk, pos);
    build_voxel(pos, chunk, culled, ctx);
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
            let me = data.get_block(pos);
            let other = other.get_block(other_pos);
            if !is_face_culled(me, other) {
                build(pos)
            }
        }
    }
}

/// Builds the quad instance for a singel voxel.
fn build_voxel(pos: LocalPos, chunk: &Chunk, culled: CulledFaces, ctx: &mut ChunkBuildContext) {
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
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::X,
                    texture: side as u32,
                });
            }
            if !culled.contains(CulledFaces::NEG_X) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::NEG_X,
                    texture: side as u32,
                });
            }
            if !culled.contains(CulledFaces::Y) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::Y,
                    texture: top as u32,
                });
            }
            if !culled.contains(CulledFaces::NEG_Y) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::NEG_Y,
                    texture: bottom as u32,
                });
            }
            if !culled.contains(CulledFaces::Z) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::Z,
                    texture: side as u32,
                });
            }
            if !culled.contains(CulledFaces::NEG_Z) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::NEG_Z,
                    texture: side as u32,
                });
            }
        }
        BlockAppearance::Liquid(surface) => {
            if !culled.contains(CulledFaces::Y) {
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::Y | QuadFlags::OFFSET_1,
                    texture: surface as u32,
                });
                buffer.push(QuadInstance {
                    flags: base_flags | QuadFlags::NEG_Y | QuadFlags::OFFSET_7,
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

/// Builds a single face of a block.
fn build_single_face_side(
    pos: LocalPos,
    chunk: &Chunk,
    flags: QuadFlags,
    ctx: &mut ChunkBuildContext,
) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { side, .. } => {
            buffer.push(QuadInstance {
                flags,
                texture: side as u32,
            });
        }
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}

/// Builds a single face of a block.
fn build_single_face_top(
    pos: LocalPos,
    chunk: &Chunk,
    flags: QuadFlags,
    ctx: &mut ChunkBuildContext,
) {
    let block = chunk.get_block(pos);
    let metadata = chunk.get_appearance(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { top, .. } => {
            buffer.push(QuadInstance {
                flags: flags | QuadFlags::Y,
                texture: top as u32,
            });
        }
        BlockAppearance::Liquid(surface) => {
            buffer.push(QuadInstance {
                flags: flags | QuadFlags::OFFSET_1 | QuadFlags::Y,
                texture: surface as u32,
            });
            buffer.push(QuadInstance {
                flags: flags | QuadFlags::NEG_Y | QuadFlags::OFFSET_7,
                texture: surface as u32,
            });
        }
        BlockAppearance::Flat(texture) => {
            // SAFETY:
            //  The block apperance is `Flat`.
            let face = unsafe { metadata.flat };

            if face == Face::Y {
                buffer.push(QuadInstance {
                    flags: flags | QuadFlags::OVERLAY | QuadFlags::Y,
                    texture: texture as u32,
                });
            }
        }
    }
}

/// Builds a single face of a block.
fn build_single_face_bottom(
    pos: LocalPos,
    chunk: &Chunk,
    flags: QuadFlags,
    ctx: &mut ChunkBuildContext,
) {
    let block = chunk.get_block(pos);

    let buffer = match block.info().visibility {
        BlockVisibility::SemiOpaque | BlockVisibility::Opaque => &mut ctx.opaque_quads,
        BlockVisibility::Invisible | BlockVisibility::Transparent => &mut ctx.transparent_quads,
    };

    match block.info().appearance {
        BlockAppearance::Invisible => (),
        BlockAppearance::Regular { bottom, .. } => buffer.push(QuadInstance {
            flags: flags | QuadFlags::NEG_Y,
            texture: bottom as u32,
        }),
        BlockAppearance::Liquid(_) => (),
        BlockAppearance::Flat(_) => (),
    }
}
