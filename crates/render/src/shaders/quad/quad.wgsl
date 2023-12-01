// Rust counterpart: `src/shaders/common.rs`
struct FrameUniforms {
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    resolution: vec2<f32>,
    _padding: vec2<u32>,
}

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

// The uniform data that's written once per chunk.
struct ChunkUniforms {
    // The position of the chunk in world-space coordinates.
    position: vec3<i32>,
}

@group(1) @binding(0)
var<uniform> chunk: ChunkUniforms;

// The instance data provided by the instance buffer.
struct Instance {
    @location(0) flags: u32,
    @location(1) texture: u32,
}

// The structure that's interpolated accross the trangles
// generated by the vertex shader.
struct Interpolator {
    // The position of the vertex in clip-space coordinates.
    @builtin(position) position: vec4<f32>,
    // The texture coordinates of the vertex.
    @location(0) tex_coords: vec2<f32>,
    // The index of the texture to use.
    @location(1) @interpolate(flat) tex_index: u32,
    // The normal of the vertex.
    @location(2) @interpolate(flat) normal: vec3<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: Instance,
) -> Interpolator {
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

    var TEX_COORDS: array<vec2<f32>, 4> = array(
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0),
        vec2(1.0, 0.0),
    );

    var NORMALS: array<vec3<f32>, 6> = array(
        vec3(1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, -1.0),
    );

    // Deconstruct the flags into local coordinates.
    // The full description of the flags is in the `src/gfx/shaders/quad.rs` file.
    let face: u32 = instance.flags & 7u;
    let rotate_90: u32 = (instance.flags >> 3u) & 1u;
    let rotate_180: u32 = (instance.flags >> 4u) & 1u;
    let mirror_x: u32 = (instance.flags >> 5u) & 1u;
    let mirror_y: u32 = (instance.flags >> 6u) & 1u;
    let local_x: u32 = (instance.flags >> 7u) & 31u;
    let local_y: u32 = (instance.flags >> 12u) & 31u;
    let local_z: u32 = (instance.flags >> 17u) & 31u;
    let offset: u32 = (instance.flags >> 22u) & 7u;

    let normal = NORMALS[face];

    // The position of the vertex relative to the voxel, origin.
    let vertex_pos = VERTICES[face * 4u + vertex_index] - normal * f32(offset)/8.0;
    // The position of the voxel within its chunk.
    let chunk_local = vec3<i32>(i32(local_x), i32(local_y), i32(local_z));
    // The position of the vertex in world-space coordinates.
    let world_pos = vec3<f32>(32 * chunk.position + chunk_local) + vertex_pos;

    // OPTIMIZE:
    //  Create an array of matrices that represent the different
    //  rotations and mirroring operations already pre-computed. This allows
    //  during a single matrix multiplication (2d) instead of four ifs.
    var tex_coords = TEX_COORDS[vertex_index];
    if rotate_90 != 0u {
        tex_coords = vec2(tex_coords.y, 1.0 - tex_coords.x);
    }
    if rotate_180 != 0u {
        tex_coords = vec2(1.0 - tex_coords.x, 1.0 - tex_coords.y);
    }
    if mirror_x != 0u {
        tex_coords = vec2(1.0 - tex_coords.x, tex_coords.y);
    }
    if mirror_y != 0u {
        tex_coords = vec2(tex_coords.x, 1.0 - tex_coords.y);
    }

    var output: Interpolator;
    output.position = frame.projection * frame.view * vec4(world_pos, 1.0);
    output.tex_coords = tex_coords;
    output.tex_index = instance.texture;
    output.normal = normal;
    return output;
}

@group(2) @binding(0)
var texture_atlas: texture_2d_array<f32>;
@group(2) @binding(1)
var texture_atlas_sampler: sampler;

const LIGHT_DIRECTION: vec3<f32> = vec3<f32>(1.3, -1.8, 1.5);
const LIGHT_INTENCITY: f32 = 0.2;

@fragment
fn fs_main(input: Interpolator) -> @location(0) vec4<f32> {
    let base_color = textureSample(
        texture_atlas,
        texture_atlas_sampler,
        input.tex_coords,
        input.tex_index,
    );
    let light = (1.0 - LIGHT_INTENCITY) + LIGHT_INTENCITY * max(0.0, dot(input.normal, LIGHT_DIRECTION));

    return vec4<f32>(base_color.rgb * light, base_color.a);
}
