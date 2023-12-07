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

// The instance data provided by the instance buffer.
struct Instance {
    @location(0) flags: u32,
    @location(1) color: u32,
    @location(2) position: vec2<f32>,
    @location(3) size: vec2<f32>,
}

// The structure that's interpolated accross the trangles
// generated by the vertex shader.
struct Interpolator {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) tex_index: u32,
    @location(2) @interpolate(flat) color: u32,
}

@vertex
fn vs_main(in: Instance, @builtin(vertex_index) vertex_index: u32) -> Interpolator {
    var VERTICES: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let vertex_pos = VERTICES[vertex_index];

    let pos = in.position + vertex_pos * in.size;

    var out: Interpolator;
    out.position = vec4<f32>(2.0 * pos.x / frame.resolution.x - 1.0, 1.0 - 2.0 * pos.y / frame.resolution.y, 0.0, 1.0);
    out.tex_coords = vertex_pos;
    out.tex_index = in.flags & 0x7Fu;
    out.color = in.color;
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
var font_atlas: texture_2d_array<f32>;
@group(1) @binding(1)
var font_atlas_sampler: sampler;

@fragment
fn fs_main(in: Interpolator) -> @location(0) vec4<f32> {
    if textureSample(font_atlas, font_atlas_sampler, in.tex_coords, in.tex_index).r < 0.5 {
        discard;
    }

    return unpack_color(in.color);
}
