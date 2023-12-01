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

// The vertex data expected by the shader.
struct Instance {
    // The start position of the line, in world space.
    @location(0) start: vec3<f32>,
    // The width of the line.
    @location(1) width: f32,
    // The end position of the line, in world space.
    @location(2) end: vec3<f32>,
    // Some flags associated with the vertex.
    @location(3) flags: u32,
    // The color associated with the vertex.
    @location(4) color: vec4<f32>,
}

// The position of the vertex after the vertex shader has been run.
struct Interpolator {
    // The clip-space position of the vertex.
    @builtin(position) position: vec4<f32>,
    // The color of the vertex.
    @location(0) @interpolate(flat) color: vec4<f32>,
    // The distance of the vertex from the camera.
    @location(1) dist_to_camera: f32,
    // The start position of the line in clip space.
    @location(2) @interpolate(flat) start: vec2<f32>,
    // The end position of the line in clip space.
    @location(3) @interpolate(flat) end: vec2<f32>,
}

// This flag indicates that the line should be drawn above everything else (i.e. depth = 0.0).
const FLAG_ABOVE: u32 = 1u;

// Computes the clip-space position of the vertex required to draw a line starting at `start` and
// finishing at `end`, with a width of `width`.
//
// # Arguments
//
// - `start` - The start position of the line, in clip-space.
// - `end` - The end position of the line, in clip-space.
// - `width` - The width of the line.
// - `index` - The index of the vertex to compute.
//
// # Returns
//
// This function returns the clip-space position of the vertex at the provided index.
fn compute_line_quad(start: vec4<f32>, end: vec4<f32>, width: f32, index: u32) -> vec4<f32> {
    // Convert the width to clip-space.
    let radius = vec2<f32>(width / frame.resolution.x, width / frame.resolution.y);

    // Offset the line position based on the vertex index.
    let line_dir = normalize(end.xy - start.xy);
    let line_normal = vec2<f32>(line_dir.y, -line_dir.x);
    var offsets: array<vec2<f32>, 4> = array(
        (line_normal - line_dir) * radius,
        (-line_normal - line_dir) * radius,
        (line_normal + line_dir) * radius,
        (-line_normal + line_dir) * radius,
    );

    // Calculate the position of the vertex in clip-space.
    var points = array(start, end);

    // Note: the `max(w, 0.0)` is a hack to make the line look better. I definitely messed up
    // the math here, but it's good enough for now.
    // For context, this is supposed to apply the perspective divide to the line offset.
    // OPTIMIZE: do this properly.
    return points[index >> 1u] + vec4(max(points[index >> 1u].w, 0.0) * offsets[index], 0.0, 0.0);
}

@vertex
fn vs_main(in: Instance, @builtin(vertex_index) vertex_index: u32) -> Interpolator {
    // Compute the clip-space position of the start and end position.
    var start = frame.projection * frame.view * vec4<f32>(in.start, 1.0);
    var end = frame.projection * frame.view * vec4<f32>(in.end, 1.0);

    // Transform those positions into a quad that includes the whole line.
    var clip_space = compute_line_quad(start, end, in.width, vertex_index);

    // Apply the flags.
    if (in.flags & FLAG_ABOVE) != 0u {
        clip_space.z = 0.0;
    }

    // I don't think this is doing what I think it's doing.
    // OPTIMIZE: do this properly.
    let dist_to_camera = 0.0;

    var out: Interpolator;
    out.position = clip_space;
    out.color = in.color;
    out.dist_to_camera = dist_to_camera;
    out.start = start.xy;
    out.end = end.xy;
    return out;
}

@fragment
fn fs_main(in: Interpolator) -> @location(0) vec4<f32> {
    var out_color = in.color;

    // fade out the color based on the depth
    // out_color.a *= 1.0 / (1.0 + in.dist_to_camera * in.dist_to_camera * 0.0005);

    return out_color;
}