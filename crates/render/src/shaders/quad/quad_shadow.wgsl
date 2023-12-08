// Rust counterpart: `src/shaders/common.rs`
struct FrameUniforms {
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    light_transform: mat4x4<f32>,
    resolution: vec2<f32>,
    fog_density: f32,
    fog_distance: f32,
    fog_color: u32,
    sky_color: u32,
    flags: u32,
    milliseconds: u32,
    sun_direction: vec3<f32>,
    fog_height: f32,
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

// Returns a number between 0.0 and 1.0 that wraps around every `millis` milliseconds.
fn periodic_mod(millis: u32) -> f32 {
    return f32(frame.milliseconds % millis) / f32(millis);
}

const TAU: f32 = 6.28318530718;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: Instance,
) -> @builtin(position) vec4<f32> {
    var VERTICES: array<vec3<f32>, 24> = array<vec3<f32>, 24>(
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

    var TEX_COORDS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 1.0),
        vec2(1.0, 0.0),
    );

    var NORMALS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
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
    let occluded_top: u32 = (instance.flags >> 25u) & 1u;
    let occluded_bottom: u32 = (instance.flags >> 26u) & 1u;
    let occluded_left: u32 = (instance.flags >> 27u) & 1u;
    let occluded_right: u32 = (instance.flags >> 28u) & 1u;
    let overlay: u32 = (instance.flags >> 29u) & 1u;
    let liquid: u32 = (instance.flags >> 30u) & 1u;

    let normal = NORMALS[face];

    // The position of the vertex relative to the voxel, origin.
    let vertex_pos = VERTICES[face * 4u + vertex_index] - normal * f32(offset)/8.0 - normal * f32(overlay) * 0.999;
    // The position of the voxel within its chunk.
    let chunk_local = vec3<i32>(i32(local_x), i32(local_y), i32(local_z));
    // The position of the vertex in world-space coordinates.
    var world_pos = vec3<f32>(32 * chunk.position + chunk_local) + vertex_pos;

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

    if liquid != 0u {
        world_pos.y += (-1.0/8.0) + cos(TAU * periodic_mod(4000u) + world_pos.x * 0.2) * (2.0/8.0);
    }

    return frame.light_transform * vec4(world_pos, 1.0);
}
