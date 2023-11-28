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
    /// The position of the camera in world-space coordinates.
    position: Vec3,
    /// The yaw of the camera, in radians.
    yaw: f32,
    /// The pitch of the camera, in radians.
    pitch: f32,
    /// The aspect ratio of the output display.
    aspect_ratio: f32,
}

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
            * dt;

        self.position.y += vertical_movement_input * Self::FLY_SPEED * dt;
    }

    /// Computes the matrix that transforms world-space coordinates into clip-space coordinates.
    pub fn matrix(&self) -> Mat4 {
        let forward = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch) * Vec3::Z;
        let perspective =
            Mat4::perspective_lh(Self::FOV_Y.to_radians(), self.aspect_ratio, 0.1, 100.0);
        let view = Mat4::look_to_lh(self.position, forward, Vec3::Y);
        perspective * view
    }
}
