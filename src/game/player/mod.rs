mod camera;
use std::sync::Arc;

use bns_render::data::RenderData;
use bns_render::Gpu;
use bns_worldgen_structure::{Structure, StructureEdit};
pub use camera::*;

mod hud;
pub use hud::*;

use bns_app::{Ctx, KeyCode, MouseButton};
use bns_core::{BlockId, Chunk, ChunkPos, Face};

use glam::{IVec3, Vec2, Vec3};

use crate::assets::Assets;
use crate::world::{QueryResult, World};

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
    is_underwater: bool,
}

impl Player {
    /// Creates a new [`Player`] instance.
    pub fn new(gpu: Arc<Gpu>, position: Vec3) -> Self {
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

            looking_at: None,
            max_reach: 8.0,

            hud: Hud::new(gpu),

            structure_block: None,

            is_underwater: false,
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
        bns_core::utility::chunk_pos_of(self.position)
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

        self.looking_at = world
            .query_line(self.position, self.camera.view.look_at(), self.max_reach)
            .ok()
            .map(|q| LookingAt::from_query(&q, self.position));

        if ctx.just_pressed(MouseButton::Left) {
            if let Some(looking_at) = self.looking_at {
                world.set_block(looking_at.world_pos, BlockId::Air);
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
                    world.set_block(target, material);
                }
            }
        }

        if ctx.pressing(KeyCode::KeyT) {
            self.position = Vec3::new(u16::MAX as f32, 0.0, 0.0);
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

        self.is_underwater = world
            .get_block(bns_core::utility::world_pos_of(self.position))
            .is_some_and(|b| b == BlockId::Water);
    }

    /// Returns whether the player's head is underwater.
    #[inline]
    pub fn is_underwater(&self) -> bool {
        self.is_underwater
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

    let min = a.min(b);
    let max = a.max(b);

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let pos = IVec3::new(x, y, z);
                if let Some(block) = world.get_block(pos) {
                    if !matches!(block.id(), BlockId::Air | BlockId::StructureBlock) {
                        edits.push(StructureEdit {
                            position: pos - min,
                            block,
                        });
                    }
                }
            }
        }
    }

    Structure {
        edits,
        bounds: max - min,
        name: None,
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
