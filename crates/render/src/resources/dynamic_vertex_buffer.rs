use std::any::type_name;
use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::Arc;

use bytemuck::NoUninit;

use crate::{Gpu, Vertices};

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
    pub fn new(gpu: Arc<Gpu>, initial_size: u32) -> Self {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(type_name::<T>()),
            mapped_at_creation: false,
            size: initial_size as u64 * size_of::<T>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            gpu,
            len: 0,
            buffer,
            _marker: PhantomData,
        }
    }

    /// Replaces the content of the buffer with the provided data.
    ///
    /// The length of the buffer is updated.
    pub fn replace(&mut self, data: &[T])
    where
        T: NoUninit,
    {
        let cap = self.buffer.size() / size_of::<T>() as u64;

        if data.len() as u64 > cap {
            self.buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(type_name::<T>()),
                mapped_at_creation: false,
                size: data.len() as u64 * size_of::<T>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        }

        self.gpu
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        self.len = data.len() as u32;
    }
}

impl<T> Vertices for DynamicVertexBuffer<T> {
    type Vertex = T;

    #[inline]
    fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..self.len as u64 * size_of::<T>() as u64)
    }
}
