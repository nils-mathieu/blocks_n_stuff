//! Provides GPU-managed resources.

mod dynamic_vertex_buffer;
pub use dynamic_vertex_buffer::*;

mod texture;
pub use texture::*;

use std::marker::PhantomData;

/// A slice into a [`VertexBuffer`].
#[derive(Clone, Copy)]
pub struct VertexBufferSlice<'a, T> {
    /// The buffer slice.
    pub(crate) buffer: wgpu::BufferSlice<'a>,
    /// The number of vertices in the slice.
    pub(crate) len: u32,
    /// The marker that includes the type of the vertices.
    pub(crate) marker: PhantomData<&'a [T]>,
}
