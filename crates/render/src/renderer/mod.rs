use std::sync::Arc;

use wgpu::TextureFormat;

use crate::shaders::fog::FogPipeline;
use crate::shaders::line::LinePipeline;
use crate::shaders::quad::QuadPipeline;
use crate::shaders::skybox::SkyboxPipeline;
use crate::shaders::text::TextPipeline;
use crate::Gpu;

pub use crate::shaders::common::TextureAtlasConfig;
use crate::shaders::ui_atlas_sprite::UiAtlasSpritePipeline;
use crate::shaders::ui_sprite::UiSpritePipeline;

mod render;

/// A target on which things can be rendered.
#[derive(Clone, Copy, Debug)]
pub struct RenderTarget<'a> {
    /// A view into the target texture.
    ///
    /// This texture must have the `RENDER_ATTACHMENT` usage.
    pub(crate) view: &'a wgpu::TextureView,
}

/// The static configuration of the [`Renderer`].
///
/// The configuration options of this struct are not expected to change during the lifetime of the
/// created renderer.
///
/// If any of those need to change, the whole [`Renderer`] needs to be re-created.
#[derive(Clone, Debug)]
pub struct RendererConfig {
    /// The format of the output image of the renderer.
    ///
    /// Providing a [`RenderTarget`] that has an output format different from this one will likely
    /// result in a panic.
    pub output_format: TextureFormat,
}

/// Contains the state required to render things using GPU resources.
pub struct Renderer {
    /// A reference to the GPU.
    gpu: Arc<Gpu>,

    /// The pipeline responsible for rendering the skybox.
    skybox_pipeline: SkyboxPipeline,
    /// The pipeline responsible for rendering quads.
    quad_pipeline: QuadPipeline,
    /// The pipeline responsible for rendering lines.
    line_pipeline: LinePipeline,

    /// The pipeline responsible for rendering fog.
    fog_pipeline: FogPipeline,

    /// The pipeline responsible for rendering text.
    text_pipeline: TextPipeline,
    /// The pipeline responsible for rendering sprites in the UI.
    ui_sprite_pipeline: UiSpritePipeline,
    /// The pipeline responsible for rendering sprites in the UI using the global texture atlas.
    ui_atlas_sprite_pipeline: UiAtlasSpritePipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, config: RendererConfig) -> Self {
        let quad_pipeline = QuadPipeline::new(&gpu, config.output_format);
        let skybox_pipeline = SkyboxPipeline::new(&gpu, config.output_format);
        let line_pipeline = LinePipeline::new(&gpu, config.output_format);
        let fog_pipeline = FogPipeline::new(&gpu, config.output_format);
        let text_pipeline = TextPipeline::new(&gpu, config.output_format);
        let ui_sprite_pipeline = UiSpritePipeline::new(&gpu, config.output_format);
        let ui_atlas_sprite_pipeline = UiAtlasSpritePipeline::new(&gpu, config.output_format);

        Self {
            gpu,
            quad_pipeline,
            skybox_pipeline,
            line_pipeline,
            fog_pipeline,
            text_pipeline,
            ui_sprite_pipeline,
            ui_atlas_sprite_pipeline,
        }
    }

    /// Returns a reference to the underlying [`Gpu`] instance.
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }
}
