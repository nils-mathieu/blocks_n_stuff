use bns_app::{Ctx, KeyCode};
use glam::{Mat4, Quat, Vec3, Vec4};

use super::player::Camera;

/// Contains the current state of the sun.
pub struct Sun {
    /// The direction of the light.
    ///
    /// This vector should always be normalized.
    direction: Vec3,

    /// The current time of day.
    ///
    /// This is used to determine the position of the sun. When nothing particular happens in the
    /// game, this number increases by 1 every millisecond.
    time: u64,
}

impl Sun {
    /// Creates a new [`Sun`] instance.
    pub fn new() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 1.5).normalize(),
            time: 0,
        }
    }

    /// Returns the current direction of the sun.
    #[inline]
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Ticks this [`Sun`] instance.
    pub fn tick(&mut self, ctx: &mut Ctx) {
        if ctx.pressing(KeyCode::KeyU) {
            self.time += ctx.since_last_tick().as_millis() as u64 * 50;
        } else {
            self.time += ctx.since_last_tick().as_millis() as u64;
        }

        let sub_day = (self.time % 600000) as f32 / 600000.0;
        self.direction = Quat::from_rotation_y(sub_day * std::f32::consts::TAU)
            * Vec3::new(0.0, -1.0, 1.5).normalize();
    }

    /// Returns the matrix of the light.
    pub fn matrix(&self, camera_pos: Vec3, camera: &Camera) -> Mat4 {
        let points = camera.frustum_quad(
            camera.projection.near(),
            (camera.projection.far() / 2.0).clamp(32.0, 64.0),
        );

        let view = Mat4::look_to_lh(Vec3::ZERO, self.direction, Vec3::Y);

        let (min, max) = points
            .iter()
            .map(|&p| view * (camera_pos + p).extend(1.0))
            .map(Vec4::truncate)
            .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(min, max), point| {
                (min.min(point), max.max(point))
            });

        const PADDING: f32 = 30.0;

        Mat4::orthographic_lh(
            min.x - PADDING,
            max.x + PADDING,
            min.y - PADDING,
            max.y + PADDING,
            min.z - PADDING,
            max.z + PADDING,
        ) * view
    }
}
