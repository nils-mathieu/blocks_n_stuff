use std::sync::Arc;

use pollster::FutureExt;
use winit::window::Window;

/// Represents an open connection with a Graphics Processing Unit.
pub struct Gpu {
    /// The instance that was created for the GPU.
    ///
    /// This instance contains the global state of the rendering API.
    pub instance: wgpu::Instance,
    /// The GPU adapter that was selected for use with the application.
    pub adapter: wgpu::Adapter,
    /// The open connection with the GPU device selected.
    pub device: wgpu::Device,
    /// The queue used to submit commands to the GPU.
    pub queue: wgpu::Queue,
}

impl Gpu {
    /// Creates a new [`Gpu`] instance.
    pub fn new(window: Arc<Window>) -> (Arc<Self>, wgpu::Surface<'static>) {
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance
            .create_surface(window)
            .expect("failed to create a surface for the provided window");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .expect("failed to find a suitable GPU adapter");
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .block_on()
            .expect("failed to establish a connection with the selected GPU device");

        (
            Arc::new(Self {
                instance,
                adapter,
                device,
                queue,
            }),
            surface,
        )
    }
}
