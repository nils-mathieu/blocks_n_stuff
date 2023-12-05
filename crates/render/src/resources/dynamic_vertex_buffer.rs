use std::any::type_name;
use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::Arc;

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::{Gpu, VertexBufferSlice};

/// A dynamic vertex buffer that can be written-to by the CPU.
pub struct DynamicVertexBuffer<T> {
    /// The GPU that owns the buffer.
    gpu: Arc<Gpu>,

    /// The buffer that stores the data.
    buffer: wgpu::Buffer,
    /// The number of `T`s that are currently stored in the buffer.
    len: u32,

    _marker: PhantomData<[T]>,
}

impl<T> DynamicVertexBuffer<T> {
    /// Creates a new [`DynamicVertexBuffer`] instance.
    #[profiling::function]
    pub fn new(gpu: Arc<Gpu>, initial_size: u32) -> Self {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(type_name::<T>()),
            mapped_at_creation: false,
            size: initial_size as u64 * size_of::<T>() as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        Self {
            gpu,
            len: 0,
            buffer,
            _marker: PhantomData,
        }
    }

    /// Creates a new [`DynamicVertexBuffer`] instance with the provided data.
    #[profiling::function]
    pub fn new_with_data(gpu: Arc<Gpu>, data: &[T]) -> Self
    where
        T: NoUninit,
    {
        let buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                contents: bytemuck::cast_slice(data),
                label: Some(type_name::<T>()),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

        Self {
            gpu,
            len: data.len() as u32,
            buffer,
            _marker: PhantomData,
        }
    }

    /// Returns the number of `T`s that are currently stored in the buffer.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Clears the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Extends the buffer with the provided data.
    #[profiling::function]
    pub fn extend(&mut self, data: &[T])
    where
        T: NoUninit,
    {
        let cap = self.buffer.size() / size_of::<T>() as wgpu::BufferAddress;
        let new_len = self.len + data.len() as u32;

        if new_len as wgpu::BufferAddress > cap {
            let new_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(type_name::<T>()),
                mapped_at_creation: false,
                size: new_len as wgpu::BufferAddress * size_of::<T>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

            // Request a copy of the old buffer into the new one.
            self.gpu
                .temp_command_encoder()
                .lock()
                .copy_buffer_to_buffer(
                    &self.buffer,
                    0,
                    &new_buffer,
                    0,
                    self.len as wgpu::BufferAddress * size_of::<T>() as wgpu::BufferAddress,
                );

            self.buffer = new_buffer;
        }

        self.gpu.queue.write_buffer(
            &self.buffer,
            self.len as wgpu::BufferAddress * size_of::<T>() as wgpu::BufferAddress,
            bytemuck::cast_slice(data),
        );

        self.len = new_len;
    }

    /// Returns a [`VertexBufferSlice`] that can be used to render the contents of this buffer.
    #[inline]
    pub fn slice(&self) -> VertexBufferSlice<T> {
        VertexBufferSlice {
            buffer: self.buffer.slice(..self.len as u64 * size_of::<T>() as u64),
            len: self.len,
            marker: PhantomData,
        }
    }
}
