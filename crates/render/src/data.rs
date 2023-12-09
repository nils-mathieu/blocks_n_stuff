use crate::shaders::quad::Quads;
use crate::{Gpu, Texture, VertexBufferSlice};

pub use crate::color::Color;
pub use crate::shaders::common::{FrameFlags, FrameUniforms};
pub use crate::shaders::line::{LineFlags, LineInstance};
pub use crate::shaders::quad::{ChunkUniforms, QuadFlags, QuadInstance};
pub use crate::shaders::text::{CharacterFlags, CharacterInstance, CharacterInstanceCursor};
pub use crate::shaders::ui_atlas_sprite::AtlasSprite;
pub use crate::shaders::ui_sprite::Sprite;

/// An UI element to draw.
///
/// UI elements are drawn in the order they are declared.
pub enum Ui<'a> {
    /// Some text lements.
    Text(VertexBufferSlice<'a, CharacterInstance>),
    /// A sprite (or a collection of sprites that share the same texture).
    Sprite {
        /// The sprite instances to draw.
        instances: VertexBufferSlice<'a, Sprite>,
        /// The texture to use for the sprites.
        texture: &'a Texture,
    },
    /// A sprite that's sampled from the global texture atlas (the one used for quads).
    AtlasSprite(VertexBufferSlice<'a, AtlasSprite>),
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

    /// Whether fog should be enabled.
    pub fog_enabled: bool,
    /// Whether shadows should be enabled.
    pub shadows_enabled: bool,
}

impl<'res> RenderData<'res> {
    /// Creates a new [`RenderData`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            uniforms: FrameUniforms::default(),
            quads: Quads::new(gpu),
            lines: Vec::new(),
            ui: Vec::new(),
            fog_enabled: true,
            shadows_enabled: true,
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
            fog_enabled: true,
            shadows_enabled: true,
        }
    }
}
