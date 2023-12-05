use std::sync::Arc;

use crate::{Gpu, RenderTarget};

pub use wgpu::{PresentMode, TextureFormat};

/// The configuration of a [`Surface`].
#[derive(Clone, Debug)]
pub struct SurfaceConfig {
    /// The width of the surface.
    pub width: u32,
    /// The height of the surface.
    pub height: u32,
    /// The present mode of the surface.
    pub present_mode: PresentMode,
}

/// Stores information about a [`Surface`].
#[derive(Clone, Debug)]
pub struct SurfaceInfo {
    /// The format of the surface.
    pub format: TextureFormat,
}

/// A surface on which it is possible to draw stuff.
pub struct Surface<'window> {
    /// A reference to the GPU that was used to create this surface.
    ///
    /// This is required to create new resources on the GPU.
    gpu: Arc<Gpu>,
    /// The configuration of the output surface on which the GPU is rendering.
    config: SurfaceConfig,
    /// Information about the surface.
    info: SurfaceInfo,
    /// The surface on which this [`Renderer`] is rendering.
    surface: wgpu::Surface<'window>,
    /// A boolean that is set when the configuration of the surface changes.
    ///
    /// When this boolean is set, the surface should be reconfigured.
    config_dirty: bool,

    //
    // The following fields are simply used to keep track of the capabilities of the surface
    // in order to construct a valid instance `SurfaceConfiguration` when the surface needs
    // to be re-configured.
    //
    alpha_mode: wgpu::CompositeAlphaMode,
}

impl<'w> Surface<'w> {
    /// Creates a new [`Surface`] from the provided window.
    ///
    /// # Panics
    ///
    /// This function panics if no GPU is found to render on, or if the rendering API cannot be
    /// initialized.
    ///
    /// # Remarks
    ///
    /// On web, this function must be polled by the web runtime in order to work properly.
    pub async fn new<W>(window: W) -> Self
    where
        W: 'w
            + wgpu::WasmNotSendSync
            + raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle,
    {
        bns_log::trace!("initiating a connection with the GPU...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window)
            .expect("GPU api not available");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("failed to find an appropriate GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_limits: wgpu::Limits::default(),
                    required_features: wgpu::Features::empty(),
                    label: Some("GPU Device"),
                },
                None,
            )
            .await
            .expect("failed to establish a connection with the selected GPU");

        bns_log::info!("established a connection with the GPU!");
        bns_log::info!("GPU: {}", adapter.get_info().name);

        let config = surface
            .get_default_config(&adapter, 0, 0)
            .expect("the selected GPU is not compatible with the surface");

        bns_log::info!("surface format: {:?}", config.format);
        bns_log::info!("present mode: {:?}", config.present_mode);

        #[allow(clippy::arc_with_non_send_sync)]
        let gpu = Arc::new(Gpu::new(device, queue));

        Self {
            gpu,
            config: SurfaceConfig {
                width: 0,
                height: 0,
                present_mode: config.present_mode,
            },
            info: SurfaceInfo {
                format: config.format,
            },
            surface,
            config_dirty: false,

            alpha_mode: config.alpha_mode,
        }
    }

    /// Returns an exclusive reference to the [`SurfaceConfig`] of this [`Surface`].
    ///
    /// This function may be used to change how the surface is configured, including its size
    /// and presentation mode.
    ///
    /// # Remarks
    ///
    /// Calling this function will cause the surface to re-create itself. If you are not sure
    /// whether the surface needs to be updated, consider gating this call behind a condition.
    #[inline]
    pub fn config_mut(&mut self) -> &mut SurfaceConfig {
        self.config_dirty = true;
        &mut self.config
    }

    /// Returns a shared reference to the [`SurfaceConfig`] of this [`Surface`].
    #[inline]
    pub fn config(&self) -> &SurfaceConfig {
        &self.config
    }

    /// Returns information about the [`Surface`].
    #[inline]
    pub fn info(&self) -> &SurfaceInfo {
        &self.info
    }

    /// Returns a reference to the underlying [`Gpu`] object.
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Acquires a new image from the surface.
    ///
    /// # Remarks
    ///
    /// On some platform, not presenting the returned image will cause the presentation engine
    /// to fail to acquire new images (usually after a timeout).
    ///
    /// # Errors
    ///
    /// This function can fail in two cases:
    ///
    /// 1. The swapchain image cannot be acquired because of an API error. In that case, this
    ///    function panics.
    ///
    /// 2. The swapchain image cannot be acquired because the surface is out of date compared to
    ///    its target window. In that case `None` is returned.
    ///
    /// Additionally, if a timeout is reached, `None` is returned as well. The behavior to
    /// adopt in both case is the same.
    pub fn acquire_image(&mut self) -> Option<Frame> {
        // If the surface configuration has changed since the last call to this function, we
        // need to re-configure the surface.
        if self.config_dirty {
            self.config_dirty = false;
            self.surface.configure(
                &self.gpu.device,
                &wgpu::SurfaceConfiguration {
                    alpha_mode: self.alpha_mode,
                    format: self.info.format,
                    width: self.config.width,
                    height: self.config.height,
                    present_mode: self.config.present_mode,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: Vec::new(),
                },
            );
        }

        // Actually acquire the image.
        // This function is responsible for blocking until an image is available in the swapchain.
        let texture = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Timeout) => {
                return None;
            }
            Err(err) => panic!("failed to acquire surface texture: {err}"),
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Some(Frame { view, texture })
    }
}

/// A frame in flight.
pub struct Frame {
    /// A view into the texture.
    ///
    /// This field must be dropped *before* `texture`.
    view: wgpu::TextureView,
    /// The texture that we're rendering to.
    texture: wgpu::SurfaceTexture,
}

impl Frame {
    /// Returns the [`RenderTarget`] of this frame.
    #[inline]
    pub fn target(&self) -> RenderTarget {
        RenderTarget { view: &self.view }
    }

    /// Present this frame to the [`Surface`].
    #[inline]
    pub fn present(self) {
        self.texture.present();
    }
}
