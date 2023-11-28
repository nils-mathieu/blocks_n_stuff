use super::helpers::UniformBufferLayout;
use super::render_data::FrameUniforms;
use super::Gpu;

pub mod quad;

/// The resources required to create the shaders.
///
/// Those resources do *not* depend on anything other than the GPU itself, meaning that it is
/// possible to create them before the [`Surface`] is even created.
pub struct RenderResources {
    /// The uniform buffer layout that's used to describe the uniform buffers that are supposed
    /// to change on every single frame.
    pub frame_uniforms: UniformBufferLayout<FrameUniforms>,

    /// A pipeline layout that assumes the presence of the following bind groups:
    ///
    /// 0. The frame uniforms.
    pub world_pipeline_layout: wgpu::PipelineLayout,
}

impl RenderResources {
    /// Creates a new [`RenderResources`] instance.
    pub fn new(gpu: &Gpu) -> Self {
        let frame_uniforms = UniformBufferLayout::new(gpu, wgpu::ShaderStages::VERTEX);
        let world_pipeline_layout = create_world_pipeline_layout(gpu, &frame_uniforms);

        Self {
            frame_uniforms,
            world_pipeline_layout,
        }
    }
}

/// Creates a [`wgpu::PipelineLayout`] as described by [`RenderResources::world_pipeline_layout`].
fn create_world_pipeline_layout(
    gpu: &Gpu,
    frame_uniforms: &UniformBufferLayout<FrameUniforms>,
) -> wgpu::PipelineLayout {
    gpu.device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("World Pipeline Layout"),
            bind_group_layouts: &[frame_uniforms.layout()],
            push_constant_ranges: &[],
        })
}
