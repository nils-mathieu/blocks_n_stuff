struct FrameUniforms {
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

// The vertex data expected by the shader.
struct Vertex {
    // The position of the vertex.
    @location(0) position: vec3<f32>,
    // Some flags associated with the vertex.
    @location(1) flags: u32,
    // The color associated with the vertex.
    @location(2) color: vec4<f32>,
}

// The position of the vertex after the vertex shader has been run.
struct Interpolator {
    // The clip-space position of the vertex.
    @builtin(position) position: vec4<f32>,
    // The color of the vertex.
    @location(0) color: vec4<f32>,
    // The distance of the vertex from the camera.
    @location(1) dist_to_camera: f32,
}

@vertex
fn vs_main(in: Vertex) -> Interpolator {
    var out: Interpolator;
    out.position = frame.projection * frame.view * vec4<f32>(in.position, 1.0);

    if (in.flags & 1u) != 0u {
        out.position.z = 0.0;
    }

    out.color = in.color;
    out.dist_to_camera = (frame.view * vec4<f32>(in.position, 1.0)).z;
    return out;
}

@fragment
fn fs_main(in: Interpolator) -> @location(0) vec4<f32> {
    var out_color = in.color;

    // fade out the color based on the depth
    out_color.a *= 1.0 / (1.0 + in.dist_to_camera * in.dist_to_camera * 0.0005);

    return out_color;
}
