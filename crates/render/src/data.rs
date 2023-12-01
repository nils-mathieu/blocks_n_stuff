use crate::shaders::quad::Quads;
use crate::Gpu;

pub use crate::shaders::common::FrameUniforms;
pub use crate::shaders::line::{LineInstance, LineVertexFlags};
pub use crate::shaders::quad::{ChunkUniforms, QuadFlags, QuadInstance};

/// The data required to render a frame.
pub struct RenderData<'res> {
    /// The frame uniforms for the frame.
    pub frame: FrameUniforms,

    /// The list of quad instances to render.
    ///
    /// This collection automatically lays the data out in a way that's compatible and efficient
    /// to send to the GPU.
    pub quads: Quads<'res>,

    /// The line instances to render.
    ///
    /// The content of this buffer is uploaded to the GPU on every frame, so if in the future
    /// we need to keep some static geometry around, we will need to use something more efficient.
    ///
    /// Right now, the lines are mainly used for debugging purposes, so this is not a problem.
    pub lines: Vec<LineInstance>,
}

impl<'res> RenderData<'res> {
    /// Creates a new [`RenderData`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            frame: FrameUniforms::default(),
            quads: Quads::new(gpu),
            lines: Vec::new(),
        }
    }

    /// Re-creates this [`RenderData`] with a potentially longer lifetime, while keeping the
    /// original allocations.
    pub fn reset<'res2>(mut self) -> RenderData<'res2> {
        self.lines.clear();

        RenderData {
            frame: self.frame,
            quads: self.quads.reset(),
            lines: self.lines,
        }
    }
}
