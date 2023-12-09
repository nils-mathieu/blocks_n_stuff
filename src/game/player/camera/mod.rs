mod perspective;
mod view;

pub use perspective::*;
pub use view::*;

use glam::Vec3;

/// Contains the current state of the camera.
pub struct Camera {
    /// The perspective projection of the camera.
    pub projection: Perspective,
    /// The view matrix of the camera.
    pub view: View,
}

impl Camera {
    /// Creates a new [`Camera`] instance.
    ///
    /// The `fov_y` is in radians.
    pub fn new(nearest_distance: f32, far_plane: f32, fov_y: f32) -> Self {
        Self {
            projection: Perspective::new(nearest_distance, 1.0, fov_y, far_plane),
            view: View::new(),
        }
    }

    /// Returns the four points of the camera's frustum quad at the provided distance from the
    /// eye position.
    ///
    /// Note that the returned points are in local-space of the camera and should be translated by
    /// the camera's actual position.
    pub fn frustum_quad(&self, min: f32, max: f32) -> [Vec3; 8] {
        let tan = (self.projection.fov_y() * 0.5).tan();

        let min_half_height = tan * min;
        let min_half_width = min_half_height * self.projection.aspect_ratio();
        let max_half_height = tan * max;
        let max_half_width = max_half_height * self.projection.aspect_ratio();

        let rotation = self.view.rotation();
        let forward = rotation * Vec3::Z;
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;

        let min_center = forward * min;
        let max_center = forward * max;

        [
            min_center + up * min_half_height - right * min_half_width,
            min_center + up * min_half_height + right * min_half_width,
            min_center - up * min_half_height + right * min_half_width,
            min_center - up * min_half_height - right * min_half_width,
            max_center + up * max_half_height - right * max_half_width,
            max_center + up * max_half_height + right * max_half_width,
            max_center - up * max_half_height + right * max_half_width,
            max_center - up * max_half_height - right * max_half_width,
        ]
    }

    /// Determines whether the provided sphere is in the camera's frustum.
    ///
    /// # Arguments
    ///
    /// - `relative_position` - The position of the sphere relative to the camera. In world space,
    ///   the formula is `sphere_position - camera_position`.
    ///
    /// - `radius` - The radius of the sphere.
    #[profiling::function]
    pub fn is_sphere_in_frustum(&self, relative_position: Vec3, radius: f32) -> bool {
        let rotation = self.view.rotation();
        let half_fov_y = self.projection.fov_y() * 0.5;
        let half_fov_x = self.projection.fov_x() * 0.5;

        // near/far planes
        let dist_z = (rotation * Vec3::Z).dot(relative_position);
        if self.projection.near() - radius > dist_z || dist_z > self.projection.far() + radius {
            return false;
        }

        // top/bottom planes
        let dist_y = (rotation * Vec3::Y).dot(relative_position);
        let dist = radius / half_fov_y.cos() + dist_z * half_fov_y.tan();
        if dist_y > dist || dist_y < -dist {
            return false;
        }

        // left/right planes
        let dist_x = (rotation * Vec3::X).dot(relative_position);
        let dist = radius / half_fov_x.cos() + dist_z * half_fov_x.tan();
        if dist_x > dist || dist_x < -dist {
            return false;
        }

        true
    }
}
