//! This module defines everything that's needed to draw stuff on the screen.

use std::sync::Arc;

use winit::window::Window;

mod helpers;
mod render_graph;
mod shaders;

mod gpu;
pub use gpu::Gpu;

pub mod render_data;

use self::helpers::UniformBuffer;
use self::render_data::{FrameUniforms, RenderData};
use self::render_graph::RenderGraph;
use self::shaders::RenderResources;

/// The renderer is responsible for using the GPU to render things on a render target.
pub struct Renderer {
    gpu: Arc<Gpu>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    resources: RenderResources,
    graph: RenderGraph,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let (width, height) = window.inner_size().into();

        let (gpu, surface) = Gpu::new(window);
        let config = surface
            .get_default_config(&gpu.adapter, width, height)
            .unwrap();
        surface.configure(&gpu.device, &config);
        let resources = RenderResources::new(&gpu);
        let graph = RenderGraph::new(&gpu, &resources, &config);

        Self {
            gpu,
            surface,
            config,
            resources,
            graph,
        }
    }

    /// Returns the [`Gpu`] instance that was used to create this [`Renderer`].
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Creates a new [`UniformBuffer`] that follows the layout of the frame uniform layout.
    pub fn create_frame_uniform_buffer(&self) -> UniformBuffer<FrameUniforms> {
        self.resources.frame_uniforms.instanciate(self.gpu.clone())
    }

    /// Notifies the renderer that the size of the window on which it is drawing has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.gpu.device, &self.config);
        self.graph.notify_resized(&self.gpu, width, height);
    }

    /// Renders a new frame on the target surface.
    pub fn render(&mut self, render_data: &RenderData) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Timeout) => return,
            Err(e) => panic!("failed to acquire next surface texture: {e}"),
        };

        let view = frame.texture.create_view(&Default::default());
        self.graph.render_on(&self.gpu, render_data, &view);

        frame.present();
    }
}
