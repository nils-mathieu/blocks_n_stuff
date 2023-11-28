use bytemuck::Pod;

/// A uniform buffer that contains an instance of `T`.
pub struct UniformBuffer {
    /// The buffer backing this [`UniformBuffer`].
    buffer: wgpu::Buffer,
    /// The layout of `bind_group`.
    bind_group_layout: wgpu::BindGroupLayout,
    /// The bind group that contains this buffer.
    bind_group: wgpu::BindGroup,
}

impl UniformBuffer {
    /// Creates a new [`UniformBuffer`] instance for the provided type.
    ///
    /// # Panics
    ///
    /// This function panics if the provided type has a size of 0.
    pub fn new_for<T>(gpu: &wgpu::Device, visibility: wgpu::ShaderStages) -> Self
    where
        T: ?Sized + Pod,
    {
        Self::new(
            gpu,
            std::any::type_name::<T>(),
            visibility,
            wgpu::BufferSize::new(std::mem::size_of::<T>() as u64)
                .expect("can't create a uniform buffer for a type that has a size of 0"),
        )
    }

    /// Returns the [`wgpu::BindGroupLayout`] that describes this buffer.
    #[inline]
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Returns the bind group that contains this buffer.
    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Creates a new [`UniformBuffer`] instance.
    pub fn new(
        gpu: &wgpu::Device,
        label: &str,
        visibility: wgpu::ShaderStages,
        size: wgpu::BufferSize,
    ) -> Self {
        let buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            mapped_at_creation: false,
            size: size.get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Writes the provided contents to the buffer.
    ///
    /// # Panics
    ///
    /// This function panics if `T` is too large to fit in the buffer.
    pub fn write<T>(&self, queue: &wgpu::Queue, contents: &T)
    where
        T: ?Sized + Pod,
    {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(contents));
    }
}
