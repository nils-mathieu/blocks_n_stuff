use glam::{Mat4, Quat, Vec3};

/// Represents a view matrix.
pub struct View {
    /// The yaw of the camera, in radians.
    yaw: f32,
    /// The pitch of the camera, in radians.
    pitch: f32,

    /// The rotation representing the orientation of the camera.
    rotation: Quat,
}

impl View {
    /// The maximum pitch value allowed.
    pub const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

    /// Creates a new [`View`] instance.
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            rotation: compute_rotation(0.0, 0.0),
        }
    }

    /// Sets the rotation of the camera.
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw.rem_euclid(std::f32::consts::TAU);
        self.pitch = pitch.clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
        self.rotation = compute_rotation(self.yaw, self.pitch);
    }

    /// An utility function that rotates the camera by the given yaw and pitch values.
    #[inline]
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.set_rotation(self.yaw + delta_yaw, self.pitch + delta_pitch);
    }

    /// Returns the current yaw value of the camera.
    #[inline]
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Returns the current pitch value of the camera.
    #[inline]
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// The rotation of the camera.
    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    /// Returns the "look at" vector of the camera, which is the direction the camera is facing.
    #[inline]
    pub fn look_at(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    /// Returns the view matrix.
    #[inline]
    pub fn matrix(&self, position: Vec3) -> Mat4 {
        Mat4::look_to_lh(position, self.look_at(), Vec3::Y)
    }
}

/// Computes the rotation of the camera from the yaw and pitch.
fn compute_rotation(yaw: f32, pitch: f32) -> Quat {
    Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch)
}
