//! A renderer abstraction that provides access to the GPU in a more convenient way.
//! The rendering abstraction used by the Blocks 'n Stuff client.

mod shaders;

mod gpu;
pub use gpu::*;

mod surface;
pub use surface::*;

mod renderer;
pub use renderer::*;

mod resources;
pub use resources::*;

pub mod data;

/// The format of the depth buffer.
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
