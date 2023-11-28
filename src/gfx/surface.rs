use std::sync::Arc;

use winit::window::Window;

use super::Gpu;

/// Represents the surface on which we are drawing.
pub struct Surface {
    /// The [`Gpu`] instance that was used to create this [`Renderer`].
    gpu: Arc<Gpu>,
    /// The surface on which the renderer is drawing.
    surface: wgpu::Surface<'static>,
    /// The current configuration of the surface/swapchain.
    surface_config: wgpu::SurfaceConfiguration,
}

impl Surface {
    /// Creates a new [`Surface`] for the provided window.
    pub fn new(gpu: Arc<Gpu>, window: Arc<Window>) -> Self {
        let surface_size = window.inner_size();
        let surface = gpu
            .instance
            .create_surface(window)
            .expect("failed to create a surface from the created window");
        let surface_config = surface
            .get_default_config(&gpu.adapter, surface_size.width, surface_size.height)
            .expect("the selected GPU adapter is not compatible with the provided surface");
        surface.configure(&gpu.device, &surface_config);

        Self {
            gpu,
            surface_config,
            surface,
        }
    }

    /// The format of the surface.
    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    /// Notifies the [`Renderer`] that the size of the window on which it is drawing has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface
            .configure(&self.gpu.device, &self.surface_config);
    }

    /// Acquires the next available image from the underlying swapchain.
    ///
    /// If the image is not ready yet, this function will block until it is.
    pub fn acquire_next_image(&self) -> wgpu::SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("failed to acquire the next image from the swapchain")
    }
}
