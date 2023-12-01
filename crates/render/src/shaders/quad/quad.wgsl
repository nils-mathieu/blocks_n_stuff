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

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

// The uniform data that's written once per chunk.
struct ChunkUniforms {
    // The position of the chunk in world-space coordinates.
    position: vec3<i32>,
}

@group(1) @binding(0)
var<uniform> chunk: ChunkUniforms;

// The structure that's interpolated accross the trangles
// generated by the vertex shader.
struct Interpolator {
    // The position of the vertex in clip-space coordinates.
    @builtin(position) position: vec4<f32>,
    // The texture coordinates of the vertex.
    @location(0) tex_coords: vec2<f32>,
    // The index of the texture to use.
    @location(1) @interpolate(flat) tex_index: u32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) instance: u32,
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

    // OPTIMIZE: not really an optimization but it would be nice to use constants here.

    // Deconstruct the instance ID into local coordinates.
    // The full description of the flags is in the `src/gfx/shaders/quad.rs` file.
    // OPTIMIZE: avoid the shifts simply by masking and checking if it's not zero.
    let face: u32 = instance & 7u;
    let rotate_90: u32 = (instance >> 3u) & 1u;
    let rotate_180: u32 = (instance >> 4u) & 1u;
    let mirror_x: u32 = (instance >> 5u) & 1u;
    let mirror_y: u32 = (instance >> 6u) & 1u;
    let local_x: u32 = (instance >> 7u) & 31u;
    let local_y: u32 = (instance >> 12u) & 31u;
    let local_z: u32 = (instance >> 17u) & 31u;
    let tex_index: u32 = (instance >> 22u) & 1023u;

    // The position of the vertex relative to the voxel, origin.
    // OPTIMIZE: move the `face` field four bits to the left to avoid
    // this multiplication.
    let vertex_pos = VERTICES[face * 4u + vertex_index];
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
    output.tex_index = tex_index;
    return output;
}

@group(2) @binding(0)
var texture_atlas: texture_2d_array<f32>;
@group(2) @binding(1)
var texture_atlas_sampler: sampler;

@fragment
fn fs_main(input: Interpolator) -> @location(0) vec4<f32> {
    return textureSample(
        texture_atlas,
        texture_atlas_sampler,
        input.tex_coords,
        input.tex_index,
    );
}