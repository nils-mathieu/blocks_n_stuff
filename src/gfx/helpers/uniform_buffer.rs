use std::marker::PhantomData;
use std::sync::Arc;

use bytemuck::Pod;

use crate::gfx::Gpu;

/// A wrapper around a [`wgpu::BindGroupLayout`] that includes a single entry that is suitable
/// for a uniform buffer containing an instance of `T`.
pub struct UniformBufferLayout<T: ?Sized> {
    /// The GPU that owns the bind group layout.
    gpu: Arc<Gpu>,
    /// The layout of the bind group.
    layout: wgpu::BindGroupLayout,
    /// The type that's expected to be stored in the uniform buffer.
    _marker: PhantomData<T>,
}

impl<T> UniformBufferLayout<T> {
    /// Creates a new [`UniformBufferLayout`] instance.
    pub fn new(gpu: Arc<Gpu>, visibility: wgpu::ShaderStages) -> Self {
        Self {
            layout: create_uniform_group_layout(
                &gpu.device,
                std::any::type_name::<T>(),
                visibility,
                wgpu::BufferSize::new(std::mem::size_of::<T>() as _)
                    .expect("can't create a UniformBufferLayout for a zero-sized type"),
            ),
            gpu,
            _marker: PhantomData,
        }
    }

    /// Returns the raw [`wgpu::BindGroupLayout`] that this [`UniformBufferLayout`] wraps.
    #[inline]
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// Instanciate a [`UniformBuffer`] that follows the layout of this [`UniformBufferLayout`].
    pub fn instanciate(&self, device: &wgpu::Device) -> UniformBuffer<T> {
        let (buffer, bind_group) = create_uniform_buffer(
            device,
            std::any::type_name::<T>(),
            wgpu::BufferSize::new(std::mem::size_of::<T>() as _)
                .expect("can't create a UniformBufferLayout for a zero-sized type"),
            &self.layout,
        );

        UniformBuffer {
            gpu: self.gpu.clone(),
            _marker: PhantomData,
            bind_group,
            buffer,
        }
    }
}

/// The implementation of [`UniformBufferLayout::new`] that has no generic parameters
/// to avoid monomorphization costs.
fn create_uniform_group_layout(
    device: &wgpu::Device,
    label: &str,
    visibility: wgpu::ShaderStages,
    size: wgpu::BufferSize,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            count: None,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: Some(size),
                ty: wgpu::BufferBindingType::Uniform,
            },
            visibility,
        }],
    })
}

/// The implementation of [`UniformBufferLayout::instanciate`] that has no generic parameters
/// to avoid monomorphization costs.
fn create_uniform_buffer(
    device: &wgpu::Device,
    label: &str,
    size: wgpu::BufferSize,
    layout: &wgpu::BindGroupLayout,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        mapped_at_creation: false,
        size: size.get(),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    (buffer, bind_group)
}

/// A uniform buffer that contains arbitrary data.
///
/// This is mainly a convenience wrapper around a [`wgpu::Buffer`] associated with a
/// [`wgpu::BindGroup`].
pub struct UniformBuffer<T> {
    /// The GPU that owns the buffer.
    gpu: Arc<Gpu>,
    /// The buffer backing this [`UniformBuffer`].
    buffer: wgpu::Buffer,
    /// The bind group that contains this buffer.
    bind_group: wgpu::BindGroup,
    /// The layout of the bind group that contains this buffer.
    _marker: PhantomData<T>,
}

impl<T> UniformBuffer<T> {
    /// Returns the raw [`wgpu::Buffer`] that this [`UniformBuffer`] wraps.
    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Writes the provided type to the buffer.
    ///
    /// # Panics
    ///
    /// This function panics if the size of `value` is larger than the size of the buffer.
    pub fn write(&self, value: &T)
    where
        T: Pod,
    {
        self.gpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }
}
