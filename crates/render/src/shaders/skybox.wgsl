#include <frame_uniforms.wgsl>

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

// The structure that's interpolated accross the trangles
// generated by the vertex shader.
struct Interpolator {
    // The position of the vertex in clip-space coordinates.
    @builtin(position) position: vec4<f32>,
    // The direction that the camera is facing in world-space coordinates.
    @location(0) eye_direction: vec3<f32>,
}

fn extract_rotation_scale(m: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(
        vec3<f32>(m[0].xyz),
        vec3<f32>(m[1].xyz),
        vec3<f32>(m[2].xyz),
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> Interpolator {
    let uv = vec2<f32>(f32(vertex_index & 1u), f32(vertex_index >> 1u)) * 2.0 - 1.0;

    var out: Interpolator;
    out.position = vec4<f32>(uv, 0.0, 1.0);
    out.eye_direction = transpose(extract_rotation_scale(frame.view)) * (frame.inverse_projection * vec4<f32>(uv, 0.0, 1.0)).xyz;
    return out;
}

// The color of the sky.
//
// This color is specifically used at very high altitudes, where the sky is
// the most visible.
const HIGH_SKY_COLOR: vec3<f32> = vec3<f32>(0.2, 0.6, 0.9);

// The color the sky in low altitudes.
const LOW_SKY_COLOR: vec3<f32> = vec3<f32>(0.6, 0.6, 1.0);

/// The color of the ground.
const GROUND_COLOR: vec3<f32> = vec3<f32>(0.2, 0.3, 0.4);

// "how fast" the ground color appears. The transition goes from LOW_SKY_COLOR to ground.
const GROUND_BLUR: f32 = 0.2;

@fragment
fn fs_main(
    in: Interpolator,
) -> @location(0) vec4<f32> {
    let height = in.eye_direction.y;

    if (height > 0.0) {
        return vec4<f32>(mix(LOW_SKY_COLOR, HIGH_SKY_COLOR, height), 1.0);
    } else if (height > -GROUND_BLUR) {
        return vec4<f32>(mix(LOW_SKY_COLOR, GROUND_COLOR, -height / GROUND_BLUR), 1.0);
    } else {
        return vec4<f32>(GROUND_COLOR, 1.0);
    }
}
