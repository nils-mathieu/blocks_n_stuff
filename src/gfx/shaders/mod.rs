use super::render_data::{ChunkUniforms, FrameUniforms};
use super::Gpu;

pub mod quad;

/// The resources required to create the shaders.
///
/// Those resources do *not* depend on anything other than the GPU itself, meaning that it is
/// possible to create them before the [`Surface`](super::Surface) is even created.
pub struct RenderResources {
    /// The bind group layout that will be used to bind the frame uniforms.
    ///
    /// Those uniform should be written/bound to the GPU once per frame to update the content
    /// of the frame uniforms.
    ///
    /// This is expected to store an instance of [`FrameUniforms`].
    pub frame_layout: wgpu::BindGroupLayout,
    /// The bind group layout that will be used to bind the chunk uniforms.
    ///
    /// Those uniforms are supposed to be updated once per chunk draw call.
    ///
    /// This is expected store a [`ChunkUniforms`] instance.
    pub chunk_layout: wgpu::BindGroupLayout,
    /// A pipeline layout that assumes the presence of the following bind groups:
    ///
    /// 0. The frame uniforms.
    /// 1. The chunk uniforms.
    pub world_pipeline_layout: wgpu::PipelineLayout,
}

impl RenderResources {
    /// Creates a new [`RenderResources`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        let frame_layout = create_frame_layout(gpu);
        let chunk_layout = create_chunk_layout(gpu);
        let world_pipeline_layout = create_world_pipeline_layout(gpu, &frame_layout, &chunk_layout);

        Self {
            frame_layout,
            chunk_layout,
            world_pipeline_layout,
        }
    }
}

fn create_frame_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<FrameUniforms>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        })
}

fn create_chunk_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Chunk Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: true,
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<ChunkUniforms>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        })
}

fn create_world_pipeline_layout(
    gpu: &Gpu,
    frame_uniforms: &wgpu::BindGroupLayout,
    chunk_uniforms: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    gpu.device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("World Pipeline Layout"),
            bind_group_layouts: &[frame_uniforms, chunk_uniforms],
            push_constant_ranges: &[],
        })
}
