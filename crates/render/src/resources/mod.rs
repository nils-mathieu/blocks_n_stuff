//! Provides GPU-managed resources.

mod dynamic_vertex_buffer;
pub use dynamic_vertex_buffer::*;

/// A collection of vertices currently living on the GPU.
#[allow(clippy::len_without_is_empty)]
pub trait Vertices {
    /// The vertex type of the collection.
    type Vertex;

    /// The number of vertices in the collection.
    fn len(&self) -> u32;

    /// A slice over the vertices in the collection.
    ///
    /// This slice is expected to have a size greater than or equal to the value returned by
    /// [`len`] (multiplied by the size of `Self::Vertex`).
    ///
    /// [`len`]: Self::len
    fn slice(&self) -> wgpu::BufferSlice;
}
