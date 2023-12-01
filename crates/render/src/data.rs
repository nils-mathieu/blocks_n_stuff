use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::shaders::quad::Quads;
use crate::{Gpu, Vertices};

pub use crate::shaders::quad::{ChunkUniforms, QuadInstance};

// OPTIMIZE: Figure out which of those fields we really need.
/// The uniform data that is uploaded to the GPU once per frame.
#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FrameUniforms {
    /// Converts view-space coordinates to clip-space coordinates.
    pub projection: Mat4,
    /// The inverse of `projection`.
    pub inverse_projection: Mat4,
    /// Converts world-space coordinates to view-space coordinates.
    pub view: Mat4,
    /// The inverse of `view`.
    pub inverse_view: Mat4,
    /// The resolution of the render target.
    pub resolution: Vec2,
    /// Some additional padding for the struct.
    pub _padding: [u32; 2],
}

bitflags! {
    /// Some flags associated with a [`LineVertex`].
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct LineVertexFlags: u32 {
        /// Whether the line should appear above all geometry in the world.
        const ABOVE = 1 << 0;
    }
}

unsafe impl Zeroable for LineVertexFlags {}
unsafe impl Pod for LineVertexFlags {}

/// A vertex that's used to construct a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct LineInstance {
    // Note on the layout:
    //  Right now the width is seaprate from the flags because we had a free padding to fill, but
    //  if needed, it might go into the flags as a bitfield.
    /// The start position of the vertex, in world space.
    pub start: Vec3,
    /// The width of the line.
    pub width: f32,
    /// The end position of the vertex, in world space.
    pub end: Vec3,
    /// Some flags associated with the line.
    pub flags: LineVertexFlags,
    /// The color of the vertex.
    pub color: Vec4,
}

/// The data required to render a frame.
///
/// An instance of this type can be created using the [`RenderDataStorage`] type.
pub struct RenderData<'res> {
    /// The frame uniforms for the frame.
    pub(crate) frame_uniforms: FrameUniforms,

    /// Allows building a list of quad instance buffers to draw.
    pub(crate) quads: Quads<'res>,

    /// The line instances to render.
    ///
    /// The content of this buffer is uploaded to the GPU on every frame, so if in the future
    /// we need to keep some static geometry around, we will need to use something more efficient.
    ///
    /// Right now, the lines are mainly used for debugging purposes, so this is not a problem.
    pub(crate) line_instances: Vec<LineInstance>,
}

impl<'res> RenderData<'res> {
    /// Creates a new [`RenderData`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            frame_uniforms: FrameUniforms::default(),
            quads: Quads::new(gpu),
            line_instances: Vec::new(),
        }
    }

    /// Re-creates this [`RenderData`] with a potentially longer lifetime, while keeping the
    /// original allocations.
    pub fn reset<'res2>(self) -> RenderData<'res2> {
        RenderData {
            frame_uniforms: self.frame_uniforms,
            quads: self.quads.reset(),
            line_instances: self.line_instances,
        }
    }

    /// Registers a new instance buffer of [`QuadInstance`]s.
    pub fn add_quad_instances(
        &mut self,
        chunk: &ChunkUniforms,
        buffer: &'res impl Vertices<Vertex = QuadInstance>,
    ) {
        self.quads.add_quad_buffer(chunk, buffer);
    }

    /// Set the [`FrameUniforms`] instance for the frame.
    #[inline]
    pub fn frame_uniforms(&mut self, value: FrameUniforms) {
        self.frame_uniforms = value;
    }

    /// Adds a line to the gizmos list.
    pub fn gizmos_line(&mut self, line: LineInstance) {
        self.line_instances.push(line);
    }

    /// Adds a new axis-aligned bounding box to the gizmos list.
    pub fn gizmos_aabb(
        &mut self,
        min: Vec3,
        max: Vec3,
        color: Vec4,
        width: f32,
        flags: LineVertexFlags,
    ) {
        use glam::vec3;

        let base = LineInstance {
            width,
            flags,
            color,
            start: Vec3::ZERO,
            end: Vec3::ZERO,
        };

        // OPTIMZE: make sure that the vector is directly written to memory and not copied
        // from stack.

        self.line_instances.extend_from_slice(&[
            // Lower face
            LineInstance {
                start: vec3(min.x, min.y, min.z),
                end: vec3(max.x, min.y, min.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, min.y, min.z),
                end: vec3(max.x, min.y, max.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, min.y, max.z),
                end: vec3(min.x, min.y, max.z),
                ..base
            },
            LineInstance {
                start: vec3(min.x, min.y, max.z),
                end: vec3(min.x, min.y, min.z),
                ..base
            },
            // Upper face
            LineInstance {
                start: vec3(min.x, max.y, min.z),
                end: vec3(max.x, max.y, min.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, max.y, min.z),
                end: vec3(max.x, max.y, max.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, max.y, max.z),
                end: vec3(min.x, max.y, max.z),
                ..base
            },
            LineInstance {
                start: vec3(min.x, max.y, max.z),
                end: vec3(min.x, max.y, min.z),
                ..base
            },
            // Vertical edges
            LineInstance {
                start: vec3(min.x, min.y, min.z),
                end: vec3(min.x, max.y, min.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, min.y, min.z),
                end: vec3(max.x, max.y, min.z),
                ..base
            },
            LineInstance {
                start: vec3(max.x, min.y, max.z),
                end: vec3(max.x, max.y, max.z),
                ..base
            },
            LineInstance {
                start: vec3(min.x, min.y, max.z),
                end: vec3(min.x, max.y, max.z),
                ..base
            },
        ]);
    }
}
