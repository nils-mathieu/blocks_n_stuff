use std::borrow::Cow;
use std::mem::size_of;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use wgpu::util::DeviceExt;
use wgpu::TextureFormat;

use crate::color::Color;

bitflags! {
    /// Some flags associated with a [`FrameUniforms`] instance.
    #[derive(Default, Debug, Clone, Copy)]
    pub struct FrameFlags: u32 {
        /// Whether the camera is currently underwater.
        const UNDERWATER = 1 << 0;
    }
}

unsafe impl Zeroable for FrameFlags {}
unsafe impl Pod for FrameFlags {}

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

/// The frame uniforms passed to shaders.
#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FrameUniforms {
    /// Converts view-space coordinates to clip-space coordinates.
    pub projection: Mat4,
    /// The inverse of `projection`.
    pub inverse_projection: Mat4,
    /// Converts world-space coordinates to view-space coordinates.
    pub view: Mat4,
    /// The inverse of `view`.
    pub inverse_view: Mat4,
    /// The resolution of the render target.
    pub resolution: Vec2,
    /// The density of the fog.
    ///
    /// The higher the value, the faster the fog will fade in.
    pub fog_density: f32,
    /// The distance at which the fog start taking effect.
    pub fog_distance: f32,
    /// The color of the fog.
    pub fog_color: Color,
    /// The color of the sky.
    pub sky_color: Color,
    /// Some flags associated with this frame.
    pub flags: FrameFlags,
    /// The number of milliseconds elapsed since the beginning of the game.
    pub milliseconds: u32,
}

/// Some resources commonly used through the renderer.
pub struct CommonResources {
    /// The depth buffer texture.
    pub depth_buffer: wgpu::TextureView,
    /// A bind group that includes the depth buffer.
    pub depth_buffer_bind_group: wgpu::BindGroup,
    /// The bind group layout used to bind the depth buffer to the shaders.
    pub depth_buffer_layout: wgpu::BindGroupLayout,
    /// A non-filtering sampler that can be used to sample pixels from a texture.
    pub pixel_sampler: wgpu::Sampler,
    /// The bind group layout used to bind the texture atlas to the shaders.
    pub texture_atlas_layout: wgpu::BindGroupLayout,
    /// The bind group of the texture atlas (created from the `texture_atlas_layout`).
    pub texture_atlas_bind_group: wgpu::BindGroup,
    /// The buffer responsible for storing the frame uniforms.
    pub frame_uniforms_buffer: wgpu::Buffer,
    /// The bind group layout used to bind the frame uniforms to the shaders.
    pub frame_uniforms_layout: wgpu::BindGroupLayout,
    /// The bind group (created from the `frame_uniforms_layout`) that includes the buffer
    /// responsible for storing the frame uniforms.
    pub frame_uniforms_bind_group: wgpu::BindGroup,
    /// A linear sampler that can be used to sample pixels from a texture.
    pub linear_sampler: wgpu::Sampler,
    /// The bind group layout used to create textures.
    pub texture_layout: wgpu::BindGroupLayout,
}

impl CommonResources {
    /// Creates a new [`CommonResources`] instance from the provided GPU.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let pixel_sampler = create_pixel_sampler(device);
        let texture_atlas_layout = create_texture_atlas_layout(device);
        let texture_atlas_bind_group = create_texture_atlas(
            device,
            queue,
            &texture_atlas_layout,
            &TextureAtlasConfig::dummy::<64>(),
            &pixel_sampler,
        );
        let frame_uniforms_layout = create_frame_uniforms_layout(device);
        let (frame_uniforms_buffer, frame_uniforms_bind_group) =
            create_frame_uniforms_buffer(device, &frame_uniforms_layout);
        let depth_buffer_layout = create_depth_buffer_layout(device);
        let linear_sampler = create_linear_sampler(device);
        let (depth_buffer, depth_buffer_bind_group) =
            create_depth_buffer(device, &depth_buffer_layout, &linear_sampler, 1, 1);
        let texture_layout = create_texture_layout(device);

        Self {
            pixel_sampler,
            texture_atlas_layout,
            texture_atlas_bind_group,
            frame_uniforms_layout,
            frame_uniforms_bind_group,
            depth_buffer,
            depth_buffer_bind_group,
            depth_buffer_layout,
            frame_uniforms_buffer,
            linear_sampler,
            texture_layout,
        }
    }

    /// Updates the texture atlas with the provided configuration.
    pub fn set_texture_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &TextureAtlasConfig,
    ) {
        self.texture_atlas_bind_group = create_texture_atlas(
            device,
            queue,
            &self.texture_atlas_layout,
            config,
            &self.pixel_sampler,
        );
    }

    /// Notifies this [`CommonResources`] that the render target has been resized.
    pub fn notify_resized(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (self.depth_buffer, self.depth_buffer_bind_group) = create_depth_buffer(
            device,
            &self.depth_buffer_layout,
            &self.linear_sampler,
            width,
            height,
        );
    }
}

fn create_frame_uniforms_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Frame Uniform Buffer"),
        mapped_at_creation: false,
        size: size_of::<FrameUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Frame Uniform Bind Group"),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    (buffer, bind_group)
}

fn create_pixel_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
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

fn create_texture_atlas_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    config: &TextureAtlasConfig,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    let texture = device.create_texture_with_data(
        queue,
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

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

fn create_depth_buffer_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Depth Buffer Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                visibility: wgpu::ShaderStages::FRAGMENT,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                count: None,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                visibility: wgpu::ShaderStages::FRAGMENT,
            },
        ],
    })
}

fn create_linear_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Regular Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    })
}

fn create_depth_buffer(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
) -> (wgpu::TextureView, wgpu::BindGroup) {
    let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Depth Buffer Bind Group"),
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

    (view, bind_group)
}

fn create_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
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
