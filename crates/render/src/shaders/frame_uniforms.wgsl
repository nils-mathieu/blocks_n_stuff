// Contains the uniform data that's meant to be updated every frame.
//
// The Rust counterpart of this type is located at `src/data.rs`.
struct FrameUniforms {
    // The projection matrix used to convert coordinates from world-space to clip-space.
    projection: mat4x4<f32>,
    // The inverse matrix of `projection`.
    inverse_projection: mat4x4<f32>,
    // The view matrix used to convert coordinates from world-space to view-space.
    view: mat4x4<f32>,
    // The inverse matrix of `view`.
    inverse_view: mat4x4<f32>,
    // The resolution of the screen.
    resolution: vec2<f32>,

    _padding: vec2<u32>,
}
