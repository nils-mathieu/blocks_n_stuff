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

        // Don't actually reconfigure the surface if the size is 0.
        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return;
        }

        self.surface
            .configure(&self.gpu.device, &self.surface_config);
    }

    /// Acquires the next available image from the underlying swapchain.
    ///
    /// If the image is not ready yet, this function will block until it is.
    ///
    /// # Errors
    ///
    /// This function may return [`None`] in either of two cases:
    ///
    /// 1. The image could not be aquired after a timeout.
    ///
    /// 2. The surface is outdated. This usually occurs because the window has been minimized and
    ///    doing about anything with it would be invalid.
    pub fn acquire_next_image(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(texture) => Some(texture),
            Err(wgpu::SurfaceError::Outdated) => None,
            Err(wgpu::SurfaceError::Timeout) => None,
            Err(e) => panic!("failed to acquire next image from the surface: {e}"),
        }
    }
}
