use parking_lot::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use thread_local::ThreadLocal;

/// Holds an open connection with a Graphics Processing Unit (GPU) and provides access to its
/// resources.
///
/// Specifically, this handle can be used to create new GPU resources and to submit commands to
/// the GPU, such as transfer commands to copy data from the CPU to the GPU.
pub struct Gpu {
    /// The limits that have been imposed on the GPU.
    pub(crate) limits: wgpu::Limits,

    /// The device that is used to communicate with the GPU.
    ///
    /// This is the actual open connection to the GPU. When this is dropped, the connection
    /// is closed.
    pub(crate) device: wgpu::Device,
    /// The queue used to submit commands to the GPU.
    pub(crate) queue: wgpu::Queue,

    /// Temprorary command encoders used to send commands to the GPU from multiple threads.
    ///
    /// The `ThredLocal` ensures that there will be no contention most of the time, but if the
    /// renderer needs to get the temporary command encoders to submit them, the mutex is used
    /// to syncronize access with background threads.
    #[cfg(not(target_arch = "wasm32"))]
    temp_command_encoders: ThreadLocal<Mutex<wgpu::CommandEncoder>>,
    #[cfg(target_arch = "wasm32")]
    temp_command_encoder: Mutex<wgpu::CommandEncoder>,
}

impl Gpu {
    /// Creates a new [`Gpu`] instance.
    pub(crate) fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            limits: device.limits(),
            queue,

            #[cfg(not(target_arch = "wasm32"))]
            temp_command_encoders: ThreadLocal::new(),
            #[cfg(target_arch = "wasm32")]
            temp_command_encoder: RefCell::new(device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Temporary Command Encoder"),
                },
            )),

            device,
        }
    }

    /// Returns the temporary command encoder for the current thread.
    pub(crate) fn temp_command_encoder(&self) -> &Mutex<wgpu::CommandEncoder> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.temp_command_encoders.get_or(|| {
                Mutex::new(
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Temporary Command Encoder"),
                        }),
                )
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            &self.temp_command_encoder
        }
    }

    /// Iterates over the temporary command encoders for all threads.
    #[inline]
    pub(crate) fn iter_temp_command_encoders(
        &self,
    ) -> impl Iterator<Item = &Mutex<wgpu::CommandEncoder>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.temp_command_encoders.iter()
        }

        #[cfg(target_arch = "wasm32")]
        {
            std::iter::once(&self.temp_command_encoder)
        }
    }
}
