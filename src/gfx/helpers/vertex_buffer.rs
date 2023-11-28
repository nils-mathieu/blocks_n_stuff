use std::sync::Arc;

use bytemuck::Pod;

use crate::gfx::Gpu;

/// Represents a vertex buffer containing instances of `T`.
pub struct VertexBuffer<T> {
    /// The GPU that owns the buffer.
    gpu: Arc<Gpu>,

    /// The GPU that owns the buffer.
    buffer: wgpu::Buffer,

    /// The type that's expected to be stored in the buffer.
    _marker: std::marker::PhantomData<T>,

    /// The number of `T`s stored in the buffer.
    len: wgpu::BufferAddress,
    /// The maximum number of `T`s that can be stored in the buffer.
    cap: wgpu::BufferAddress,
}

impl<T> VertexBuffer<T> {
    /// Creates a new [`VertexBuffer`] instance.
    pub fn new(gpu: Arc<Gpu>, capacity: wgpu::BufferAddress) -> Self {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: capacity * std::mem::size_of::<T>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gpu,
            buffer,
            _marker: std::marker::PhantomData,
            len: 0,
            cap: capacity,
        }
    }

    /// Returns the slice of the buffer that contains the data.
    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer
            .slice(..self.len * std::mem::size_of::<T>() as wgpu::BufferAddress)
    }

    /// Returns the number of `T`s stored in the buffer.
    #[inline]
    pub fn len(&self) -> wgpu::BufferAddress {
        self.len
    }

    /// Replaces the content of the buffer, eventually resizing it if needed.
    pub fn write(&mut self, data: &[T])
    where
        T: Pod,
    {
        let data_len = data.len() as wgpu::BufferAddress;

        // If the buffer is not large enough, resize it.
        if data_len > self.cap {
            self.buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: self.cap * std::mem::size_of::<T>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.cap = data_len;
        }

        self.len = data_len;
        self.gpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}
