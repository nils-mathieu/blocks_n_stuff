use std::sync::Arc;

use wgpu::TextureFormat;

use crate::shaders::common::CommonResources;
use crate::shaders::fog::FogPipeline;
use crate::shaders::line::LinePipeline;
use crate::shaders::quad::QuadPipeline;
use crate::shaders::skybox::SkyboxPipeline;
use crate::Gpu;

pub use crate::shaders::common::TextureAtlasConfig;

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
pub struct RendererConfig<'a> {
    /// The format of the output image of the renderer.
    ///
    /// Providing a [`RenderTarget`] that has an output format different from this one will likely
    /// result in a panic.
    pub output_format: TextureFormat,
    /// The texture atlas to use initially.
    pub texture_atlas: TextureAtlasConfig<'a>,
}

/// Contains the state required to render things using GPU resources.
pub struct Renderer {
    /// A reference to the GPU.
    gpu: Arc<Gpu>,

    /// Some resources commonly used through the renderer.
    resources: CommonResources,

    /// The pipeline responsible for rendering the skybox.
    skybox_pipeline: SkyboxPipeline,
    /// The pipeline responsible for rendering quads.
    quad_pipeline: QuadPipeline,
    /// The pipeline responsible for rendering lines.
    line_pipeline: LinePipeline,

    /// The pipeline responsible for rendering fog.
    fog_pipeline: FogPipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, config: RendererConfig) -> Self {
        let resources = CommonResources::new(&gpu, &config.texture_atlas);
        let quad_pipeline = QuadPipeline::new(&gpu, &resources, config.output_format);
        let skybox_pipeline = SkyboxPipeline::new(&gpu, &resources, config.output_format);
        let line_pipeline = LinePipeline::new(&gpu, &resources, config.output_format);
        let fog_pipeline = FogPipeline::new(&gpu, &resources, config.output_format);

        Self {
            gpu,
            resources,
            quad_pipeline,
            skybox_pipeline,
            line_pipeline,
            fog_pipeline,
        }
    }

    /// Returns a reference to the underlying [`Gpu`] instance.
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Resize the resources that this [`Renderer`] is using, targeting a [`RenderTarget`]
    /// of the provided size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resources.notify_resized(&self.gpu, width, height);
    }

    /// Re-creates the texture atlas.
    pub fn set_texture_atlas(&mut self, config: &TextureAtlasConfig) {
        self.resources.set_texture_atlas(&self.gpu, config);
    }
}
