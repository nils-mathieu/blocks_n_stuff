// Turns a 4x4 matrix into a 3x3 matrix by extracting the rotation and scale
// components.
fn extract_rotation_scale(m: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(
        vec3<f32>(m[0].xyz),
        vec3<f32>(m[1].xyz),
        vec3<f32>(m[2].xyz),
    );
}
