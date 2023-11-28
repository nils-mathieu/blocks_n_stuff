// The uniform data that's written every frame.
struct InstantUniforms {
    // The camera matrix.
    //
    // This matrix convert world-space coordinates to clip-space coordinates.
    camera: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> instant_uniforms: InstantUniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(in_vertex_index & 1u);
    let y = f32(in_vertex_index >> 1u);
    return instant_uniforms.camera * vec4<f32>(x, y, 1.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
