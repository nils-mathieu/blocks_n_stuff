use std::mem::size_of;
use std::sync::Arc;

use wgpu::TextureFormat;

use crate::data::{ChunkUniforms, FrameUniforms};
use crate::{shaders, Gpu};

/// A target on which things can be rendered.
#[derive(Clone, Copy, Debug)]
pub struct RenderTarget<'a> {
    /// A view into the target texture.
    ///
    /// This texture must have the `RENDER_ATTACHMENT` usage.
    pub(crate) view: &'a wgpu::TextureView,
}

/// The static configuration of the [`Renderer`].
///
/// The configuration options of this struct are not expected to change during the lifetime of the
/// created renderer.
///
/// If any of those need to change, the whole [`Renderer`] needs to be re-created.
#[derive(Clone, Debug)]
pub struct RendererConfig {
    /// The format of the output image of the renderer.
    ///
    /// Providing a [`RenderTarget`] that has an output format different from this one will likely
    /// result in a panic.
    pub output_format: TextureFormat,
}

/// Contains the state required to render things using GPU resources.
pub struct Renderer {
    /// A reference to the GPU.
    gpu: Arc<Gpu>,

    /// The alignment of the chunk uniforms buffer.
    pub(crate) chunk_uniforms_alignment: u32,

    /// A view into the depth buffer texture.
    depth_buffer: wgpu::TextureView,

    /// The buffer responsible for storing an instance of [`FrameUniforms`].
    frame_uniforms_buffer: wgpu::Buffer,
    /// A bind group that includes `frame_uniforms_buffer`.
    frame_uniforms_bind_group: wgpu::BindGroup,

    /// The layout of `chunk_uniforms_bind_group`.
    chunk_uniforms_layout: wgpu::BindGroupLayout,
    /// The buffer responsible for storing an instance of [`ChunkUniforms`].
    chunk_uniforms_buffer: wgpu::Buffer,
    /// A bind group that includes the `chunk_uniforms_bind_group`.
    chunk_uniforms_bind_group: wgpu::BindGroup,

    /// The pipeline responsible for rendering axis-aligned quads.
    ///
    /// More infor in [`quad::create_shader`].
    quad_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, config: RendererConfig) -> Self {
        let limits = gpu.device.limits();

        let chunk_uniforms_alignment = wgpu::util::align_to(
            size_of::<ChunkUniforms>() as u32,
            limits.min_uniform_buffer_offset_alignment,
        );

        let depth_buffer = create_depth_buffer(&gpu, 1, 1);
        let frame_uniforms_layout = create_frame_uniforms_layout(&gpu);
        let (frame_uniforms_buffer, frame_uniforms_bind_group) =
            create_frame_uniforms_buffer(&gpu, &frame_uniforms_layout);
        let chunk_uniforms_layout = create_chunks_uniforms_layout(&gpu, chunk_uniforms_alignment);
        let (chunk_uniforms_buffer, chunk_uniforms_bind_group) = create_chunks_uniforms_buffer(
            &gpu,
            &chunk_uniforms_layout,
            64 * chunk_uniforms_alignment as wgpu::BufferAddress,
            chunk_uniforms_alignment,
        );
        let pipeline_layout =
            create_pipeline_layout(&gpu, &frame_uniforms_layout, &chunk_uniforms_layout);
        let quad_pipeline =
            shaders::quad::create_shader(&gpu, &pipeline_layout, config.output_format);

        Self {
            gpu,
            depth_buffer,
            chunk_uniforms_alignment,
            frame_uniforms_buffer,
            frame_uniforms_bind_group,
            chunk_uniforms_layout,
            chunk_uniforms_bind_group,
            chunk_uniforms_buffer,
            quad_pipeline,
        }
    }

    /// Returns a reference to the underlying [`Gpu`] instance.
    #[inline]
    pub fn gpu(&self) -> &Arc<Gpu> {
        &self.gpu
    }

    /// Resize the resources that this [`Renderer`] is using, targeting a [`RenderTarget`]
    /// of the provided size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.depth_buffer = create_depth_buffer(&self.gpu, width, height);
    }
}

// Implementation of `Renderer::render`.
#[path = "render.rs"]
mod render;

/// Creates a new depth buffer texture.
fn create_depth_buffer(gpu: &Gpu, width: u32, height: u32) -> wgpu::TextureView {
    let depth_buffer = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Buffer"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    depth_buffer.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_frame_uniforms_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
                visibility: wgpu::ShaderStages::VERTEX,
            }],
        })
}

fn create_frame_uniforms_buffer(
    gpu: &Gpu,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Frame Uniform Buffer"),
        mapped_at_creation: false,
        size: size_of::<FrameUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Frame Uniform Bind Group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    (buffer, bind_group)
}

fn create_chunks_uniforms_layout(gpu: &Gpu, alignment: u32) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Chunks Uniforms Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(alignment as wgpu::BufferAddress),
                    ty: wgpu::BufferBindingType::Uniform,
                },
                visibility: wgpu::ShaderStages::VERTEX,
            }],
        })
}

fn create_chunks_uniforms_buffer(
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
    total_size: wgpu::BufferAddress,
    alignment: u32,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Chunks Uniforms Buffer"),
        mapped_at_creation: false,
        size: total_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Chunks Uniforms Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(alignment as wgpu::BufferAddress),
            }),
        }],
    });

    (buffer, bind_group)
}

fn create_pipeline_layout(
    gpu: &Gpu,
    frame_uniforms_layout: &wgpu::BindGroupLayout,
    chunk_uniforms_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    gpu.device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&frame_uniforms_layout, &chunk_uniforms_layout],
            push_constant_ranges: &[],
        })
}
