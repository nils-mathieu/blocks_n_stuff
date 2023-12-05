use wgpu::util::DeviceExt;

use crate::Gpu;

/// A GPU-managed texture.
pub struct Texture {
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Texture {
    /// Creates a new [`Texture`] instance.
    pub fn new(
        gpu: &Gpu,
        width: u32,
        height: u32,
        format: crate::TextureFormat,
        data: &[u8],
    ) -> Self {
        let res = gpu.resources.read();

        let texture = gpu.device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                dimension: wgpu::TextureDimension::D2,
                format,
                label: Some("Texture"),
                mip_level_count: 1,
                sample_count: 1,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &res.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&res.pixel_sampler),
                },
            ],
        });

        Self { bind_group }
    }
}
