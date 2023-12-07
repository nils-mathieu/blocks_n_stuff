mod camera;
use std::sync::Arc;
use std::time::Duration;

use bns_render::data::RenderData;
use bns_render::Gpu;
use bns_worldgen_structure::{Structure, StructureEdit};
pub use camera::*;

mod hud;
pub use hud::*;

mod physics;

use bns_app::{Ctx, KeyCode, MouseButton};
use bns_core::{BlockId, Chunk, ChunkPos, Face};

use glam::{IVec3, Vec2, Vec3};

use crate::assets::Assets;
use crate::world::{QueryResult, World};

use self::physics::{Collider, CollisionContext, Hit};

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
    /// The current velocity of the player.
    velocity: Vec3,
    /// The amount of air drag applied to the player.
    air_drag: f32,
    /// The amount of ground drag applied to the player when sprinting.
    ground_drag: f32,
    /// The amount of drag applied to the player when in water.
    water_drag: f32,
    /// The amount of drag applied to the player when flying.
    air_drag_flying: f32,

    /// The camera that the player uses to view the world.
    camera: Camera,

    /// A collection of chunks that the player can see from its point of view.
    ///
    /// This is a cache that's updated every time the player moves.
    chunks_in_view: Vec<ChunkPos>,

    /// The block at which the player is currently looking at.
    looking_at: Option<LookingAt>,

    /// The reach of the player, in blocks.
    max_reach: f32,

    /// The HUD displayed in front the player.
    hud: Hud,

    /// If a structure block has already been interacted with, this is the position of the first
    /// block that was selected.
    structure_block: Option<IVec3>,

    /// Whether the head of the player is currently underwater.
    is_face_underwater: bool,
    /// Whether the feet of the player are currently underwater.
    are_feet_underwater: bool,

    /// Whether the player is currently flying.
    is_flying: bool,

    /// The gravity applied to the player every frame.
    gravity: Vec3,

    /// The collider of the player.
    collider: Collider,

    /// The velocity that the player will have when jumping.
    jump_velocity: f32,

    /// Whether the player is currently on ground.
    is_on_ground: bool,

    /// The amount of air control of the player (portion of the player's acceleration
    /// that's allowed in air).
    air_control: f32,

    /// The speed at which the player swims.
    swim_speed: f32,

    /// The base FOV (vertical) of the player.
    base_fov: f32,

    /// Some state that's used to make collision detection more efficient.
    collision_context: CollisionContext,

    /// The instant of the last jump.
    last_jump_instant: Duration,
    /// The instant of the last forward input.
    last_forward_input: Duration,
}

impl Player {
    /// Creates a new [`Player`] instance.
    pub fn new(gpu: Arc<Gpu>, position: Vec3) -> Self {
        let collider_radius = 0.4;

        let render_distance = 8;
        let far_plane = render_distance_to_far_plane(render_distance);
        let base_fov = 60f32.to_radians();

        Self {
            mouse_sensitivity: 0.002,
            render_distance,
            vertical_render_distance: 6,
            speed: 50.0,
            fly_speed: 200.0,
            swim_speed: 100.0,
            sprint_factor: 3.0,
            sprinting: false,
            position,
            velocity: Vec3::ZERO,
            camera: Camera::new(0.01, far_plane, base_fov),

            chunks_in_view: Vec::new(),

            looking_at: None,
            max_reach: 8.0,

            hud: Hud::new(gpu),

            structure_block: None,

            is_face_underwater: false,
            are_feet_underwater: false,
            is_flying: false,
            gravity: Vec3::new(0.0, -50.0, 0.0),

            collider: Collider {
                height: 1.8,
                radius: collider_radius,
            },

            jump_velocity: 13.0,

            is_on_ground: false,

            air_drag: 0.99,
            ground_drag: 0.93,
            water_drag: 0.95,
            air_drag_flying: 0.9,

            air_control: 0.3,

            base_fov,

            collision_context: CollisionContext::new(),
            last_jump_instant: Duration::ZERO,
            last_forward_input: Duration::ZERO,
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

    /// Returns the position of the player's head.
    #[inline]
    pub fn head_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, self.collider.height - 0.1, 0.0)
    }

    /// Returns the chunk that the player is a part of.
    #[inline]
    pub fn position_chunk(&self) -> ChunkPos {
        ChunkPos::from_world_pos(self.position)
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

    /// Returns the position of the block that the player is currently looking at.
    #[inline]
    pub fn looking_at(&self) -> Option<LookingAt> {
        self.looking_at
    }

    /// Tick the player state.
    #[profiling::function]
    pub fn tick(&mut self, world: &mut World, ctx: &mut Ctx) {
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

        self.hud.tick(ctx);

        if ctx.just_pressed(MouseButton::Middle) {
            if let Some(looking_at) = self.looking_at {
                *self.hud.current_material_mut() = Some(looking_at.block);
                self.hud.rebuild_ui(ctx.width(), ctx.height());
            }
        }

        self.looking_at = world
            .query_line(
                self.head_position(),
                self.camera.view.look_at(),
                self.max_reach,
            )
            .ok()
            .map(|q| LookingAt::from_query(&q, self.position));

        if ctx.just_pressed(MouseButton::Left) {
            if let Some(looking_at) = self.looking_at {
                world.set_block(looking_at.world_pos, BlockId::Air.into());
            }
        }

        if ctx.just_pressed(MouseButton::Right) {
            if let Some(looking_at) = self.looking_at {
                if looking_at.block == BlockId::StructureBlock {
                    match self.structure_block.take() {
                        Some(other) => {
                            bns_log::trace!(
                                "Structure block #2 registered: {}",
                                looking_at.world_pos
                            );

                            let s = record_structure(world, other, looking_at.world_pos);
                            write_structure_file(&s);
                        }
                        None => {
                            bns_log::trace!(
                                "Structure block #1 registered: {}",
                                looking_at.world_pos
                            );

                            self.structure_block = Some(looking_at.world_pos);
                        }
                    }
                } else if let Some(material) = self.hud.current_material() {
                    let target = looking_at.world_pos + looking_at.face.normal();
                    world.set_block(target, material.into());
                }
            }
        }

        if ctx.pressing(KeyCode::KeyT) {
            self.position = Vec3::new(u16::MAX as f32, 0.0, 0.0);
        }

        let current_fov = self.camera.projection.fov_y();
        let target_fov = if self.sprinting {
            self.base_fov * 1.2
        } else {
            self.base_fov
        };
        if (current_fov - target_fov).abs() > 0.001 {
            self.camera
                .projection
                .set_fov_y(current_fov + (target_fov - current_fov) * 0.075);
        }

        // ======================================
        // Movement
        // ======================================

        let sprint_factor = if self.sprinting {
            self.sprint_factor
        } else {
            1.0
        };
        let drag = if self.is_flying {
            self.air_drag_flying
        } else if self.are_feet_underwater {
            self.water_drag
        } else if self.is_on_ground {
            self.ground_drag
        } else {
            self.air_drag
        };
        let speed = if self.is_flying {
            self.fly_speed
        } else if self.is_on_ground {
            self.speed
        } else {
            self.speed * self.air_control
        };

        if self.is_flying {
            let hdelta = Vec2::from_angle(-self.camera.view.yaw())
                .rotate(horizontal_movement_input)
                * speed
                * sprint_factor
                * ctx.delta_seconds();
            let vdelta = vertical_movement_input * self.fly_speed * ctx.delta_seconds();
            self.velocity += Vec3::new(hdelta.x, vdelta, hdelta.y);
        } else {
            // Apply gravity
            self.velocity += self.gravity * ctx.delta_seconds();

            // Apply horizontal movement.
            let hdelta = Vec2::from_angle(-self.camera.view.yaw())
                .rotate(horizontal_movement_input)
                * speed
                * sprint_factor
                * ctx.delta_seconds();

            self.velocity += Vec3::new(hdelta.x, 0.0, hdelta.y);
        }
        self.velocity *= drag;

        self.is_face_underwater = world
            .get_block(bns_core::utility::world_pos_of(self.head_position()))
            .is_some_and(|b| b == BlockId::Water);
        self.are_feet_underwater = world
            .get_block(bns_core::utility::world_pos_of(
                self.position + Vec3::new(0.0, 0.01, 0.0),
            ))
            .is_some_and(|b| b == BlockId::Water);

        if !self.is_flying {
            #[allow(clippy::collapsible_if)]
            if self.are_feet_underwater {
                if ctx.pressing(KeyCode::Space) {
                    self.velocity.y += self.swim_speed * ctx.delta_seconds();
                }
            } else if self.is_on_ground {
                if ctx.just_pressed(KeyCode::Space) {
                    self.velocity.y = self.jump_velocity;
                }
            }
        }

        if ctx.just_pressed(KeyCode::Space) {
            if self.last_jump_instant + Duration::from_millis(300) > ctx.since_startup() {
                self.is_flying = !self.is_flying;
            } else {
                self.last_jump_instant = ctx.since_startup();
            }
        }

        if ctx.just_pressed(KeyCode::KeyW) {
            if self.last_forward_input + Duration::from_millis(300) > ctx.since_startup() {
                self.sprinting = true;
            } else {
                self.last_forward_input = ctx.since_startup();
            }
        }

        self.is_on_ground = false;

        // Resolve collisions.
        let hit = self.collision_context.sweep(
            self.collider,
            &mut self.position,
            &mut self.velocity,
            ctx.delta_seconds(),
            world,
        );

        if hit.contains(Hit::NEG_Y) {
            self.is_on_ground = true;
            self.is_flying = false;
        }

        if hit.contains(Hit::HORIZONAL) {
            self.sprinting = false;
        }
    }

    /// Returns whether the player's head is underwater.
    #[inline]
    pub fn is_underwater(&self) -> bool {
        self.is_face_underwater
    }

    /// Renders the player's HUD.
    pub fn render_hud<'res>(&'res self, assets: &'res Assets, frame: &mut RenderData<'res>) {
        self.hud.render(assets, frame);
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

/// Stores information about what the player is looking at.
#[derive(Debug, Clone, Copy)]
pub struct LookingAt {
    /// The position of the block.
    pub world_pos: IVec3,
    /// The ID of the block.
    pub block: BlockId,
    /// The distance from the player to the block.
    pub distance: f32,
    /// The face of the block that the player is looking at.
    pub face: Face,
}

impl LookingAt {
    /// Creates a new [`LookingAt`] instance from a [`QueryResult`].
    pub fn from_query(query: &QueryResult, pos: Vec3) -> Self {
        Self {
            world_pos: query.world_pos,
            block: query.chunk.get_block(query.local_pos),
            distance: query.hit.distance(pos),
            face: query.face,
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

/// Record a [`Structure`] from the given world between the two given positions.
fn record_structure(world: &World, a: IVec3, b: IVec3) -> Structure {
    let mut edits = Vec::new();

    let mut origin = None;

    let min = a.min(b);
    let max = a.max(b);

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let pos = IVec3::new(x, y, z);
                if let Some(block) = world.get_block_instance(pos) {
                    if block == BlockId::StructureOriginBlock {
                        if origin.is_some() {
                            bns_log::warning!("multiple origin blocks found in structure");
                        }
                        origin = Some(pos);
                    } else if !matches!(block.id(), BlockId::Air | BlockId::StructureBlock) {
                        edits.push(StructureEdit {
                            position: pos,
                            block,
                        });
                    }
                }
            }
        }
    }

    if origin.is_none() {
        bns_log::warning!("no origin block found in structure, falling back to center");
    }

    let origin = origin.unwrap_or(IVec3::new((min.x + max.x) / 2, min.y, (min.z + max.z) / 2));

    edits.iter_mut().for_each(|x| x.position -= origin);

    Structure {
        edits: edits.into(),
        min: min - origin,
        max: max - origin,
    }
}

/// Writes the provided list of blocks to a structure file.
fn write_structure_file(structure: &Structure) {
    const FILE_NAME: &str = "structure.ron";

    bns_log::info!(
        "Writing {} blocks to '{}'...",
        structure.edits.len(),
        FILE_NAME
    );
    let s = ron::ser::to_string_pretty(structure, ron::ser::PrettyConfig::default()).unwrap();
    download_file(FILE_NAME, &s);
}

#[cfg(not(target_arch = "wasm32"))]
fn download_file(name: &str, data: &str) {
    std::fs::write(name, data).unwrap();
}

#[cfg(target_arch = "wasm32")]
fn download_file(name: &str, data: &str) {
    use web_sys::wasm_bindgen::{JsCast, JsValue};

    let string = web_sys::js_sys::Array::from_iter(&[JsValue::from_str(data)]);
    let blob = web_sys::Blob::new_with_str_sequence(&string).unwrap();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let elem = document
        .create_element("a")
        .unwrap()
        .unchecked_into::<web_sys::HtmlAnchorElement>();
    elem.set_href(&url);
    elem.set_download(name);
    elem.click();

    web_sys::Url::revoke_object_url(&url).unwrap();
}
