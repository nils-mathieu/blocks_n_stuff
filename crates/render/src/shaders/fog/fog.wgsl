// Rust counterpart: `src/shaders/common.rs`
struct FrameUniforms {
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    resolution: vec2<f32>,
    fog_factor: f32,
    fog_distance: f32,
    fog_color: u32,
    flags: u32,
    milliseconds: u32,
}

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

// The structure that's interpolated accross the trangles
// generated by the vertex shader.
struct Interpolator {
    // The position of the vertex in clip-space coordinates.
    @builtin(position) position: vec4<f32>,
    // The direction that the camera is facing in world-space coordinates.
    @location(0) eye_direction: vec3<f32>,
    // The UV coordinates of the vertex.
    @location(1) uv: vec2<f32>,
}

// Turns a 4x4 matrix into a 3x3 matrix by extracting the rotation and scale
// components.
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
    let uv = vec2<f32>(f32(vertex_index & 1u), 1.0 - f32(vertex_index >> 1u));

    var out: Interpolator;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.eye_direction = transpose(extract_rotation_scale(frame.view)) * (frame.inverse_projection * vec4<f32>(uv, 0.0, 1.0)).xyz;
    out.uv = uv;
    return out;
}


// Unpacks the provided color.
fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((color >> 24u) & 0xFFu) / 255.0,
        f32((color >> 16u) & 0xFFu) / 255.0,
        f32((color >> 8u) & 0xFFu) / 255.0,
        f32(color & 0xFFu) / 255.0,
    );
}

@group(1) @binding(0)
var depth_texture: texture_depth_2d;
@group(1) @binding(1)
var the_sampler: sampler;

fn depth_value(uv: vec2<f32>) -> f32 {
    var depth = textureSample(depth_texture, the_sampler, vec2(uv.x, -uv.y));
    let clip_space = vec4<f32>(vec2(0.0, 1.0) - uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    let view_space = frame.inverse_projection * clip_space;
    return view_space.z / view_space.w;
}

@fragment
fn fs_main(
    in: Interpolator,
) -> @location(0) vec4<f32> {
    let depth = max(0.0, depth_value(in.uv) - frame.fog_distance);
    let fog_amount = 1.0 - pow(2.0, -depth * frame.fog_factor);
    var fog_color = unpack_color(frame.fog_color);
    fog_color.a *= fog_amount;
    return fog_color;
}
