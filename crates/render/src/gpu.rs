/// Holds an open connection with a Graphics Processing Unit (GPU) and provides access to its
/// resources.
///
/// Specifically, this handle can be used to create new GPU resources and to submit commands to
/// the GPU, such as transfer commands to copy data from the CPU to the GPU.
pub struct Gpu {
    /// The device that is used to communicate with the GPU.
    ///
    /// This is the actual open connection to the GPU. When this is dropped, the connection
    /// is closed.
    pub(crate) device: wgpu::Device,
    /// The queue used to submit commands to the GPU.
    pub(crate) queue: wgpu::Queue,
}
