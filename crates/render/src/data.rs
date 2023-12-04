use crate::shaders::quad::Quads;
use crate::{Gpu, VertexBufferSlice};

pub use crate::color::Color;
pub use crate::shaders::common::FrameUniforms;
pub use crate::shaders::line::{LineInstance, LineVertexFlags};
pub use crate::shaders::quad::{ChunkUniforms, QuadFlags, QuadInstance};
pub use crate::shaders::text::{CharacterFlags, CharacterInstance, CharacterInstanceCursor};

/// An UI element to draw.
///
/// UI elements are drawn in the order they are declared.
pub enum Ui<'a> {
    /// Some text lements.
    Text(VertexBufferSlice<'a, CharacterInstance>),
}

/// The data required to render a frame.
pub struct RenderData<'res> {
    /// The frame uniforms for the frame.
    pub uniforms: FrameUniforms,

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

    /// A collection of UI elements.
    pub ui: Vec<Ui<'res>>,
}

impl<'res> RenderData<'res> {
    /// Creates a new [`RenderData`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            uniforms: FrameUniforms::default(),
            quads: Quads::new(gpu),
            lines: Vec::new(),
            ui: Vec::new(),
        }
    }

    /// Re-creates this [`RenderData`] with a potentially longer lifetime, while keeping the
    /// original allocations.
    pub fn reset<'res2>(mut self) -> RenderData<'res2> {
        self.lines.clear();
        self.ui.clear();

        // SAFETY:
        //  Any type has the same layout regardless of which lifetime it uses. No
        //  references are actually being transmuted into a potentially longer lifetime because
        //  the buffer is empty.
        let ui = unsafe { std::mem::transmute(self.ui) };

        RenderData {
            uniforms: self.uniforms,
            quads: self.quads.reset(),
            lines: self.lines,
            ui,
        }
    }
}
