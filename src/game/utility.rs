use bns_render::data::{Color, LineFlags, LineInstance};
use glam::Vec3;

/// Adds a new axis-aligned bounding box to the gizmos list.
pub fn push_aabb_lines(
    lines: &mut Vec<LineInstance>,
    min: Vec3,
    max: Vec3,
    color: Color,
    width: f32,
    flags: LineFlags,
) {
    use glam::vec3;

    let base = LineInstance {
        width,
        flags,
        color,
        start: Vec3::ZERO,
        end: Vec3::ZERO,
    };

    lines.extend_from_slice(&[
        // Lower face
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(max.x, min.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(min.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, min.y, min.z),
            ..base
        },
        // Upper face
        LineInstance {
            start: vec3(min.x, max.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, min.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, max.y, max.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        // Vertical edges
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
    ]);
}
