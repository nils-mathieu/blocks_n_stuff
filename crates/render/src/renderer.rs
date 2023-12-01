use std::borrow::Cow;
use std::mem::size_of;
use std::sync::Arc;

use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use crate::data::{FrameUniforms, LineInstance};
use crate::shaders::quad::QuadPipeline;
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
pub struct RendererConfig<'a> {
    /// The format of the output image of the renderer.
    ///
    /// Providing a [`RenderTarget`] that has an output format different from this one will likely
    /// result in a panic.
    pub output_format: TextureFormat,
    /// The texture atlas to use initially.
    pub texture_atlas: TextureAtlasConfig<'a>,
}

/// Information about a texture atlas to be created.
#[derive(Clone, Debug)]
pub struct TextureAtlasConfig<'a> {
    /// The data of the texture atlas.
    pub data: Cow<'a, [u8]>,
    /// The width of the textures in the atlas.
    pub width: u32,
    /// The height of the textures in the atlas.
    pub height: u32,
    /// The number of textures in the atlas.
    pub count: u32,
    /// The number of mip levels to generate.
    pub mip_level_count: u32,
    /// The format of the textures in the atlas.
    pub format: TextureFormat,
}

impl<'a> TextureAtlasConfig<'a> {
    /// Creates a dummy [`TextureAtlasConfig`] with the provided number of images.
    pub const fn dummy<const COUNT: usize>() -> Self {
        Self {
            data: Cow::Borrowed(&[0u8; COUNT]),
            width: 1,
            height: 1,
            count: COUNT as u32,
            mip_level_count: 1,
            format: TextureFormat::R8Unorm,
        }
    }
}

/// Contains the state required to render things using GPU resources.
pub struct Renderer {
    /// A reference to the GPU.
    gpu: Arc<Gpu>,

    /// A view into the depth buffer texture.
    depth_buffer: wgpu::TextureView,

    /// The buffer responsible for storing an instance of [`FrameUniforms`].
    frame_uniforms_buffer: wgpu::Buffer,
    /// A bind group that includes `frame_uniforms_buffer`.
    frame_uniforms_bind_group: wgpu::BindGroup,

    /// The sampler responsible for sampling pixels from the texture atlas.
    pixel_sampler: wgpu::Sampler,

    /// The texture atlas bind group layout, used to create the `texture_atlas_bind_group`.
    texture_atlas_layout: wgpu::BindGroupLayout,
    /// The bind group responsible for using the `texture_atlas`.
    texture_atlas_bind_group: wgpu::BindGroup,

    /// The pipeline responsible for the skybox.
    ///
    /// More information in [`shaders::skybox::create_shader`].
    skybox_pipeline: wgpu::RenderPipeline,

    /// The pipeline responsible for rendering lines.
    line_pipeline: wgpu::RenderPipeline,
    /// The buffer responsible for storing the line instances.
    line_instances: wgpu::Buffer,

    /// The pipeline responsible for rendering lines.
    quad_pipeline: QuadPipeline,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    pub fn new(gpu: Arc<Gpu>, config: RendererConfig) -> Self {
        let depth_buffer = create_depth_buffer(&gpu, 1, 1);
        let frame_uniforms_layout = create_frame_uniforms_layout(&gpu);
        let (frame_uniforms_buffer, frame_uniforms_bind_group) =
            create_frame_uniforms_buffer(&gpu, &frame_uniforms_layout);
        let pixel_sampler = create_pixel_sampler(&gpu);
        let texture_atlas_layout = create_texture_atlas_layout(&gpu);
        let texture_atlas_bind_group = create_texture_atlas(
            &gpu,
            &texture_atlas_layout,
            &config.texture_atlas,
            &pixel_sampler,
        );
        let quad_pipeline = QuadPipeline::new(
            &gpu,
            &frame_uniforms_layout,
            &texture_atlas_layout,
            config.output_format,
        );
        let skybox_pipeline =
            shaders::skybox::create_shader(&gpu, &frame_uniforms_layout, config.output_format);
        let line_pipeline =
            shaders::line::create_shader(&gpu, &frame_uniforms_layout, config.output_format);
        let line_vertices = create_line_instance_buffer(&gpu);

        Self {
            gpu,
            depth_buffer,
            frame_uniforms_buffer,
            frame_uniforms_bind_group,
            texture_atlas_bind_group,
            texture_atlas_layout,
            pixel_sampler,
            quad_pipeline,
            skybox_pipeline,
            line_pipeline,
            line_instances: line_vertices,
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

    /// Re-creates the texture atlas.
    pub fn set_texture_atlas(&mut self, config: &TextureAtlasConfig) {
        self.texture_atlas_bind_group = create_texture_atlas(
            &self.gpu,
            &self.texture_atlas_layout,
            config,
            &self.pixel_sampler,
        );
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
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

fn create_pixel_sampler(gpu: &Gpu) -> wgpu::Sampler {
    gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Pixel Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    })
}

fn create_texture_atlas_layout(gpu: &Gpu) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Atlas Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    visibility: wgpu::ShaderStages::FRAGMENT,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    visibility: wgpu::ShaderStages::FRAGMENT,
                },
            ],
        })
}

fn create_texture_atlas(
    gpu: &Gpu,
    layout: &wgpu::BindGroupLayout,
    config: &TextureAtlasConfig,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture = gpu.device.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: config.count,
            },
            mip_level_count: config.mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &config.data,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Texture Atlas Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    bind_group
}

fn create_line_instance_buffer(gpu: &Gpu) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Line Instance Buffer"),
        mapped_at_creation: false,
        size: 64 * size_of::<LineInstance>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    })
}
