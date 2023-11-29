use glam::{Mat4, Quat, Vec2, Vec3};
use winit::event::KeyEvent;
use winit::keyboard::KeyCode;

/// Stores the state of the camera.
#[derive(Default)]
pub struct Camera {
    /// Whether the user is currently pressing the "forward" key.
    pressing_forward: bool,
    /// Whether the user is currently pressing the "backward" key.
    pressing_backward: bool,
    /// Whether the user is currently pressing the "left" key.
    pressing_left: bool,
    /// Whether the user is currently pressing the "right" key.
    pressing_right: bool,
    /// Whether the user is currently pressing the "fly up" key.
    pressing_fly_up: bool,
    /// Whether the user is currently pressing the "fly down" key.
    pressing_fly_down: bool,
    /// Whether the camera is in "sprint" mode.
    ///
    /// In this mode, the horizontal movement is multiplied by a factor.
    sprinting: bool,
    /// The position of the camera in world-space coordinates.
    position: Vec3,
    /// The yaw of the camera, in radians.
    yaw: f32,
    /// The pitch of the camera, in radians.
    pitch: f32,
    /// The aspect ratio of the output display.
    aspect_ratio: f32,
}

// OPTIMIZE:
//  The forward vector and rotation quaternions are computed multiple times per frame when
//  they only really change when new inputs from the player are received.

impl Camera {
    /// The speed at which the camera moves, in units per second.
    pub const SPEED: f32 = 0.1;
    /// The speed at which the camera flies up/down.
    pub const FLY_SPEED: f32 = 0.1;
    /// The vertical field of view of the camera, in degrees.
    pub const FOV_Y: f32 = 60.0;
    /// The sensitivity of the mouse.
    pub const MOUSE_SENSITIVITY: f32 = 0.002;
    /// The maximum pitch value of the camera.
    pub const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
    /// The distance of the near plane from the camera.
    pub const NEAR: f32 = 0.1;
    /// The distance of the far plane from the camera.
    pub const FAR: f32 = 1000.0;
    /// The amount of speed to add when sprinting.
    pub const SPRINT_FACTOR: f32 = 4.0;

    /// Notifies the camera that the size of the output display has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// Notifies the camera that a keyboard event has been received.
    pub fn notify_keyboard(&mut self, event: &KeyEvent) {
        if event.physical_key == KeyCode::KeyW {
            self.pressing_forward = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyS {
            self.pressing_backward = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyA {
            self.pressing_left = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyD {
            self.pressing_right = event.state.is_pressed();
        } else if event.physical_key == KeyCode::Space {
            self.pressing_fly_up = event.state.is_pressed();
        } else if event.physical_key == KeyCode::ShiftLeft {
            self.pressing_fly_down = event.state.is_pressed();
        } else if event.physical_key == KeyCode::ControlLeft && event.state.is_pressed() {
            self.sprinting = !self.sprinting;
        }

        if !self.pressing_forward {
            self.sprinting = false;
        }
    }

    /// Notifies the camera that the mouse has moved.
    pub fn notify_mouse_moved(&mut self, dx: f64, dy: f64) {
        self.yaw += dx as f32 * Self::MOUSE_SENSITIVITY;
        self.pitch += dy as f32 * Self::MOUSE_SENSITIVITY;
        self.pitch = self.pitch.clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    /// Updates the state of the camera.
    pub fn tick(&mut self, dt: f32) {
        let mut movement_input = Vec2::ZERO;
        if self.pressing_forward {
            movement_input.y += 1.0;
        }
        if self.pressing_backward {
            movement_input.y -= 1.0;
        }
        if self.pressing_left {
            movement_input.x -= 1.0;
        }
        if self.pressing_right {
            movement_input.x += 1.0;
        }
        movement_input = movement_input.normalize_or_zero();

        let mut vertical_movement_input = 0.0;
        if self.pressing_fly_up {
            vertical_movement_input += 1.0;
        }
        if self.pressing_fly_down {
            vertical_movement_input -= 1.0;
        }

        self.position += Quat::from_rotation_y(self.yaw)
            * Vec3::new(movement_input.x, 0.0, movement_input.y)
            * Self::SPEED
            * (if self.sprinting {
                Self::SPRINT_FACTOR
            } else {
                1.0
            })
            * dt;

        self.position.y += vertical_movement_input * Self::FLY_SPEED * dt;
    }

    /// Returns the position of the camera.
    #[inline]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Computes the view matrix of the camera.
    pub fn view_matrix(&self) -> Mat4 {
        let forward = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch) * Vec3::Z;
        Mat4::look_to_lh(self.position, forward, Vec3::Y)
    }

    /// Computes the projection matrix of the camera.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_lh(
            Self::FOV_Y.to_radians(),
            self.aspect_ratio,
            Self::NEAR,
            Self::FAR,
        )
    }

    /// Determines whether the provided sphere is in the camera's frustum.
    ///
    /// # Arguments
    ///
    /// - `relative_position` - The position of the sphere relative to the camera. In world space,
    ///   the formula is `sphere_position - camera_position`.
    ///
    /// - `radius` - The radius of the sphere.
    pub fn is_sphere_in_frustum(&self, relative_position: Vec3, radius: f32) -> bool {
        let rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);

        let forward = rotation * Vec3::Z;
        let up = rotation * Vec3::Y;
        let right = rotation * Vec3::X;
        let half_fov_y = 0.5 * Self::FOV_Y.to_radians();
        let half_fov_x = self.aspect_ratio * half_fov_y;

        // near/far planes
        let dist_z = forward.dot(relative_position);
        if Self::NEAR - radius > dist_z || dist_z > Self::FAR + radius {
            return false;
        }

        // top/bottom planes
        let dist_y = up.dot(relative_position);
        let dist = radius / half_fov_y.cos() + dist_z * half_fov_y.tan();
        if dist_y > dist || dist_y < -dist {
            return false;
        }

        // left/right planes
        let dist_x = right.dot(relative_position);
        let dist = radius / half_fov_x.cos() + dist_z * half_fov_x.tan();
        if dist_x > dist || dist_x < -dist {
            return false;
        }

        true
    }
}
