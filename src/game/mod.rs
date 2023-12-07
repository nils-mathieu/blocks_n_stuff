//! Contains the state of the game world (not including eventual menus).

use std::sync::Arc;
use std::time::Duration;

use bns_app::{Ctx, KeyCode};
use bns_core::ChunkPos;
use bns_render::data::{ChunkUniforms, Color, FrameFlags, FrameUniforms, LineFlags, RenderData};
use bns_render::Gpu;
use bns_rng::{DefaultRng, FromRng};
use bns_worldgen_std::StandardWorldGenerator;

use glam::{Vec2, Vec3};

use self::debug::DebugThings;
use self::player::{LookingAt, Player};
use crate::assets::Assets;
use crate::world::World;

pub mod player;

mod debug;
mod utility;

/// The amount of time that must have passed before the world cleans up its unused data.
///
/// This is done to avoid cleaning up the data too often, which would be a waste of resources
/// and of time (as freeing memory may be relatively expensive in some cases).
const WORLD_CLEAN_UP_INTERVAL: Duration = Duration::from_secs(4);

/// The current state of the game.
pub struct Game {
    /// An open connection with the GPU.
    gpu: Arc<Gpu>,
    /// The state of the player currently playing the game.
    player: Player,
    /// The world that contains the block data and the background generation logic.
    world: World,
    /// The amount of time that has passed since the last time the world has cleaned
    /// up its unused data.
    since_last_cleanup: Duration,
    /// The seed that was used to create the [`World`].
    seed: u64,
    /// Some state that's only used for debugging purposes.
    debug: DebugThings,

    /// Whether or not the fog is enabled.
    fog_enabled: bool,
}

impl Game {
    /// Creates a new [`Game`] with the provided seed.
    pub fn new(gpu: Arc<Gpu>, seed: u64) -> Self {
        bns_log::info!("creating a new world with seed: {seed}");
        let generator = Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed));
        let world = World::new(gpu.clone(), generator);
        let player = Player::new(gpu.clone(), Vec3::new(0.0, 16.0, 0.0));
        let debug = DebugThings::new(gpu.clone());

        Self {
            gpu,
            player,
            world,
            since_last_cleanup: Duration::ZERO,
            seed,
            debug,
            fog_enabled: true,
        }
    }

    /// Advances the [`Game`] state by one tick.
    #[profiling::function]
    pub fn tick(&mut self, ctx: &mut Ctx) {
        self.debug.reset_overlay();

        if ctx.just_pressed(KeyCode::KeyR) {
            let seed = bns_rng::entropy();
            bns_log::info!("re-creating world with seed: {seed}");
            let generator = Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed));
            self.world = World::new(self.gpu.clone(), generator);
            self.seed = seed;
        }

        if ctx.just_pressed(KeyCode::F10) {
            self.fog_enabled = !self.fog_enabled;
        }

        self.since_last_cleanup += ctx.since_last_tick();
        if self.since_last_cleanup >= WORLD_CLEAN_UP_INTERVAL {
            self.world.request_cleanup(
                ChunkPos::from_world_pos(self.player.position()),
                self.player.render_distance() as u32 + 3,
                self.player.vertical_render_distance() as u32 + 3,
            );
            self.since_last_cleanup = Duration::ZERO;
        }

        self.player.tick(&mut self.world, ctx);
        self.player.compute_chunks_in_view();

        for &chunk_pos in self.player.chunks_in_view() {
            self.world.request_chunk(chunk_pos);
        }

        // Make sure that the chunks that are closest to the player are loaded first.
        let player_chunk = self.player.position_chunk();
        self.world
            .sort_pending_chunks(|p| -player_chunk.distance_squared(p));
        self.world.flush_pending_chunks();

        let _ = writeln!(
            self.debug.overlay_buffer(),
            "Position: {:.2} {:.2} {:.2}\n\
            Chunk: {} {} {}\n\
            Pitch: {:.2}, Yaw: {:.2}\n\
            \n\
            Loading chunks: {}\n\
            Loaded chunks: {}\n\
            Visible chunks: {}\n\
            \n\
            Looking at: {}\n\
            \n\
            Seed: {}",
            self.player.position().x,
            self.player.position().y,
            self.player.position().z,
            self.player.position_chunk().x,
            self.player.position_chunk().y,
            self.player.position_chunk().z,
            self.player.camera().view.pitch().to_degrees(),
            self.player.camera().view.yaw().to_degrees(),
            self.world.loading_chunk_count(),
            self.world.loaded_chunk_count(),
            self.player.chunks_in_view().len(),
            DisplayLookingAt(self.player.looking_at()),
            self.seed,
        );

        let _ = self.world.generator().debug_info(
            self.debug.overlay_buffer(),
            self.player.position().floor().as_ivec3(),
        );

        let _ = writeln!(self.debug.overlay_buffer());

        self.debug.tick(ctx);
    }

    /// Renders the game.
    #[profiling::function]
    pub fn render<'res>(
        &'res mut self,
        ctx: &mut Ctx,
        assets: &'res Assets,
        frame: &mut RenderData<'res>,
    ) {
        let mut fog_distance = self.player.render_distance() as f32 * 3.0;
        let mut fog_density = 0.1 / self.player.render_distance() as f32;
        let mut fog_color = Color::rgb(100, 200, 255);
        let mut sky_color = Color::rgb(150, 100, 255);
        if self.player.is_underwater() {
            fog_distance = 4.0;
            fog_density *= 24.0;
            fog_color = Color::rgb(2, 5, 30);
            sky_color = fog_color;
        }

        // Initialize the frame.
        let projection = self.player.camera().projection.matrix();
        let view = self.player.camera().view.matrix(self.player.position());
        frame.uniforms = FrameUniforms {
            inverse_projection: projection.inverse(),
            inverse_view: view.inverse(),
            view,
            projection,
            fog_distance,
            fog_density,
            resolution: Vec2::new(ctx.width() as f32, ctx.height() as f32),
            fog_color,
            sky_color,
            flags: FrameFlags::UNDERWATER,
            milliseconds: ctx.since_startup().as_millis() as u32,
        };
        frame.fog_enabled = self.fog_enabled;

        // Register the world geometry.
        let mut total_quad_count = 0;
        for &chunk_pos in self.player.chunks_in_view() {
            let Some(chunk) = self.world.get_chunk(chunk_pos) else {
                continue;
            };

            if chunk.geometry.is_empty() {
                continue;
            }

            let chunk_idx = frame.quads.register_chunk(&ChunkUniforms {
                position: chunk_pos.as_ivec3(),
            });
            if let Some(buf) = chunk.geometry.opaque_quad_instances() {
                frame.quads.register_opaque_quads(chunk_idx, buf.slice());
                total_quad_count += buf.len();
            }
            if let Some(buf) = chunk.geometry.transparent_quad_instances() {
                frame
                    .quads
                    .register_transparent_quads(chunk_idx, buf.slice());
                total_quad_count += buf.len();
            }
        }

        self.player.render_hud(assets, frame);

        // Outline the block that the player is looking at.
        if let Some(looking_at) = self.player.looking_at() {
            const PADDING: f32 = 0.01;

            utility::push_aabb_lines(
                &mut frame.lines,
                looking_at.world_pos.as_vec3() - Vec3::ONE * PADDING,
                looking_at.world_pos.as_vec3() + Vec3::ONE * (1.0 + PADDING),
                Color::WHITE,
                2.0,
                LineFlags::empty(),
            );
        }

        let _ = writeln!(
            self.debug.overlay_buffer(),
            "Render distance: {}x{}",
            self.player.render_distance(),
            self.player.vertical_render_distance()
        );
        let _ = writeln!(
            self.debug.overlay_buffer(),
            "Total quads: {}",
            total_quad_count
        );

        self.debug.render(self.player.position_chunk(), frame);
    }
}

/// A simple wrapper that implement [`std::fmt::Display`] to display
/// what the player is currently looking at.
struct DisplayLookingAt(Option<LookingAt>);

impl std::fmt::Display for DisplayLookingAt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(looking_at) = self.0 {
            write!(
                f,
                "{} {} {} ({:?}) ({} blocks)",
                looking_at.world_pos.x,
                looking_at.world_pos.y,
                looking_at.world_pos.z,
                looking_at.block,
                looking_at.distance,
            )
        } else {
            write!(f, "nothing")
        }
    }
}
