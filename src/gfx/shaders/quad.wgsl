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
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance: u32,
) -> @builtin(position) vec4<f32> {
    var VERTICES: array<vec3<f32>, 24> = array(
        // Positive X
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 1.0, 0.0),
        vec3(1.0, 0.0, 1.0),
        vec3(1.0, 1.0, 1.0),
        // Negative X
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 1.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        // Positive Y
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 1.0),
        vec3(1.0, 1.0, 0.0),
        vec3(1.0, 1.0, 1.0),
        // Negative Y
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 0.0),
        vec3(1.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0),
        // Positive Z
        vec3(1.0, 0.0, 1.0),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 1.0),
        // Negative Z
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 1.0, 0.0),
    );

    // Deconstruct the instance ID into local coordinates.
    // The full description of the flags is in the `src/gfx/shaders/quad.rs` file.
    let face: u32 = instance & 7u;
    let rotate_90: u32 = (instance >> 3u) & 1u;
    let rotate_180: u32 = (instance >> 4u) & 1u;
    let mirror_x: u32 = (instance >> 5u) & 1u;
    let mirror_y: u32 = (instance >> 6u) & 1u;
    let local_x: u32 = (instance >> 7u) & 31u;
    let local_y: u32 = (instance >> 12u) & 31u;
    let local_z: u32 = (instance >> 17u) & 31u;

    let local_pos =
        // Position of the voxel within its chunk.
        // vec3(f32(local_x), f32(local_y), f32(local_z)) +
        // Position of the vertex within the voxel.
        VERTICES[face * 4u + vertex_index];

    return instant_uniforms.camera * vec4(local_pos, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 1.0);
}
