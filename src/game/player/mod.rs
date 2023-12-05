mod camera;
pub use camera::*;

use bns_app::{Ctx, KeyCode};
use bns_core::{Chunk, ChunkPos};

use glam::{IVec3, Vec2, Vec3};

/// Contains the state of the player, including camera orientation and computed intent.
pub struct Player {
    /// The mouse sensitivity of the player.
    mouse_sensitivity: f32,
    /// The number of chunks that the player can see from its point of view.
    render_distance: i32,
    /// The vertical render distance.
    vertical_render_distance: i32,

    /// The speed at which the player moves, in blocks per second.
    ///
    /// This speed is multiplied by the sprint factor when sprinting.
    speed: f32,
    /// The speed at which the player moves up and down, in blocks per second.
    fly_speed: f32,
    /// How much the speed is multiplied by when sprinting.
    sprint_factor: f32,

    /// Whether the player is currently sprinting.
    sprinting: bool,

    /// The current position of the player.
    position: Vec3,
    /// The camera that the player uses to view the world.
    camera: Camera,

    /// A collection of chunks that the player can see from its point of view.
    ///
    /// This is a cache that's updated every time the player moves.
    chunks_in_view: Vec<ChunkPos>,
}

impl Player {
    /// Creates a new [`Player`] instance.
    pub fn new(position: Vec3) -> Self {
        let render_distance = 8;
        let far_plane = render_distance_to_far_plane(render_distance);

        Self {
            mouse_sensitivity: 0.002,
            render_distance,
            vertical_render_distance: 6,
            speed: 10.0,
            fly_speed: 20.0,
            sprint_factor: 16.0,
            sprinting: false,
            position,
            camera: Camera::new(far_plane, 60f32.to_radians()),

            chunks_in_view: Vec::new(),
        }
    }

    /// Sets the render distance of the player.
    pub fn set_render_distance(&mut self, render_distance: i32) {
        self.render_distance = render_distance;
        self.camera
            .projection
            .set_far(render_distance_to_far_plane(render_distance));
    }

    /// Returns the position of the player.
    #[inline]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Returns the camera state of the player.
    #[inline]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Returns the chunk that the player is a part of.
    #[inline]
    pub fn position_chunk(&self) -> ChunkPos {
        bns_core::utility::chunk_of(self.position)
    }

    /// Returns the current render distance of the player.
    #[inline]
    pub fn render_distance(&self) -> i32 {
        self.render_distance
    }

    /// Returns the current vertical render distance of the player.
    #[inline]
    pub fn vertical_render_distance(&self) -> i32 {
        self.vertical_render_distance
    }

    /// Returns the list of chunks that the player can see.
    #[inline]
    pub fn chunks_in_view(&self) -> &[ChunkPos] {
        &self.chunks_in_view
    }

    /// Tick the player state.
    #[profiling::function]
    pub fn tick(&mut self, ctx: &mut Ctx) {
        // ======================================
        // Controls & Events
        // ======================================

        if ctx.just_resized() {
            let aspect_ratio = ctx.width() as f32 / ctx.height() as f32;
            self.camera.projection.set_aspect_ratio(aspect_ratio);
        }

        if ctx.mouse_delta_x() != 0.0 || ctx.mouse_delta_y() != 0.0 {
            self.camera.view.rotate(
                ctx.mouse_delta_x() as f32 * self.mouse_sensitivity,
                ctx.mouse_delta_y() as f32 * self.mouse_sensitivity,
            );
        }

        if ctx.just_pressed(KeyCode::ArrowUp) && self.render_distance < 32 {
            self.set_render_distance(self.render_distance + 1);
            bns_log::trace!("render distance: {}", self.render_distance);
        }
        if ctx.just_pressed(KeyCode::ArrowDown) && self.render_distance > 1 {
            self.set_render_distance(self.render_distance - 1);
            bns_log::trace!("render distance: {}", self.render_distance);
        }

        let horizontal_movement_input = compute_horizontal_movement_input(ctx);
        let vertical_movement_input = compute_vertical_movement_input(ctx);

        if ctx.just_pressed(KeyCode::ControlLeft) && ctx.pressing(KeyCode::KeyW) {
            self.sprinting = true;
        }
        if !ctx.pressing(KeyCode::KeyW) {
            self.sprinting = false;
        }

        // ======================================
        // Movement
        // ======================================

        let sprint_factor = if self.sprinting {
            self.sprint_factor
        } else {
            1.0
        };
        let hdelta = Vec2::from_angle(-self.camera.view.yaw()).rotate(horizontal_movement_input)
            * self.speed
            * sprint_factor
            * ctx.delta_seconds();
        let vdelta = vertical_movement_input * self.fly_speed * ctx.delta_seconds();
        self.position += Vec3::new(hdelta.x, vdelta, hdelta.y);
    }

    /// Re-computes the chunks that are in view of the player.
    #[profiling::function]
    pub fn compute_chunks_in_view(&mut self) {
        const CHUNK_RADIUS: f32 = (Chunk::SIDE as f32) * 0.8660254; // sqrt(3) / 2

        self.chunks_in_view.clear();
        let center = self.position_chunk();
        for x in -self.render_distance..=self.render_distance {
            for y in -self.vertical_render_distance..=self.vertical_render_distance {
                for z in -self.render_distance..=self.render_distance {
                    if x * x + z * z > self.render_distance * self.render_distance {
                        continue;
                    }

                    let relative_chunk_pos = IVec3::new(x, y, z);
                    let relative_chunk_pos_center =
                        (relative_chunk_pos.as_vec3() + Vec3::splat(0.5)) * Chunk::SIDE as f32
                            - (self.position - center.as_vec3() * Chunk::SIDE as f32);

                    if self
                        .camera
                        .is_sphere_in_frustum(relative_chunk_pos_center, CHUNK_RADIUS)
                    {
                        self.chunks_in_view.push(center + relative_chunk_pos);
                    }
                }
            }
        }
    }
}

/// Converts a render distance measured in chunks to a far plane for the camera.
fn render_distance_to_far_plane(render_distance: i32) -> f32 {
    (render_distance as f32 + 2.0) * Chunk::SIDE as f32
}

/// Computes the movement input that the player should have along the horizontal axis.
fn compute_horizontal_movement_input(ctx: &Ctx) -> Vec2 {
    let mut input = Vec2::ZERO;

    if ctx.pressing(KeyCode::KeyW) {
        input.y += 1.0;
    }
    if ctx.pressing(KeyCode::KeyS) {
        input.y -= 1.0;
    }
    if ctx.pressing(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if ctx.pressing(KeyCode::KeyD) {
        input.x += 1.0;
    }

    input.normalize_or_zero()
}

/// Computes the movement input that the player should have along the vertical axis.
fn compute_vertical_movement_input(ctx: &Ctx) -> f32 {
    let mut input = 0.0;

    if ctx.pressing(KeyCode::Space) {
        input += 1.0;
    }
    if ctx.pressing(KeyCode::ShiftLeft) {
        input -= 1.0;
    }

    input
}