use glam::Mat4;

/// Describes a perspective projection matrix.
pub struct Perspective {
    /// The nearest any object may approach the camera.
    ///
    /// The near plane is derived from this value and the field of view.
    nearest_distance: f32,

    /// The aspect ratio of the output display.
    ///
    /// This is the width divided by the height.
    aspect_ratio: f32,

    /// The field of view of the camera, in *radians*.
    fov_y: f32,

    /// The near plane of the camera.
    ///
    /// This value should generally not be modified directly. It is computed from other values.
    near: f32,
    /// The far plane of the camera.
    far: f32,
}

impl Perspective {
    /// Creates a new [`Perspective`] instance.
    ///
    /// The near plane of the projection is computed from the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `nearest_distance` - The nearest any object may approach the camera without being clipped.
    ///
    /// * `aspect_ratio` - The aspect ratio of the output display. This is the width divided by the
    ///
    /// * `fov_y` - The vertical field of view of the camera, in radians.
    ///
    /// * `far` - The far plane of the camera. Objects further than this distance will be clipped.
    pub fn new(nearest_distance: f32, aspect_ratio: f32, fov_y: f32, far: f32) -> Self {
        let cached_near = compute_near_plane(nearest_distance, aspect_ratio, fov_y);

        Self {
            nearest_distance,
            aspect_ratio,
            fov_y,
            near: cached_near,
            far,
        }
    }

    /// Returns the vertical field of view of the projection, in radians.
    #[inline]
    pub fn fov_y(&self) -> f32 {
        self.fov_y
    }

    /// Returns the horizontal field of view of the projection, in radians.
    #[inline]
    pub fn fov_x(&self) -> f32 {
        self.fov_y * self.aspect_ratio
    }

    /// Returns the near plane of the projection.
    #[inline]
    pub fn near(&self) -> f32 {
        self.near
    }

    /// Returns the far plane of the projection.
    #[inline]
    pub fn far(&self) -> f32 {
        self.far
    }

    /// Sets the far plane of the projection.
    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    /// Sets the vertical field of view of the projection.
    pub fn set_fov_y(&mut self, fov_y: f32) {
        self.fov_y = fov_y;
        self.near = compute_near_plane(self.nearest_distance, self.aspect_ratio, fov_y);
    }

    /// Sets the aspect ratio of the projection.
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.near = compute_near_plane(self.nearest_distance, aspect_ratio, self.fov_y);
    }

    /// Returns the aspect ratio of the camera.
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Returns the projection matrix of the camera.
    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.near, self.far)
    }
}

/// Computes the ideal near plane distance from the provided parameters.
fn compute_near_plane(nearest_distance: f32, aspect_ratio: f32, fov_y: f32) -> f32 {
    // nearPlane = nearestApproachToPlayer / sqrt(1 + tan(fov/2)^2 * (aspectRatio^2 + 1)))
    let tan_fov_y = (fov_y * 0.5).tan();
    nearest_distance / (1.0 + tan_fov_y * tan_fov_y * (aspect_ratio * aspect_ratio + 1.0)).sqrt()
}
