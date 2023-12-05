//! Contains the state of the game world (not including eventual menus).

use std::sync::Arc;
use std::time::Duration;

use bns_app::{Ctx, KeyCode};
use bns_render::data::{ChunkUniforms, FrameUniforms, RenderData};
use bns_render::Gpu;
use bns_rng::{DefaultRng, FromRng};
use bns_worldgen_std::StandardWorldGenerator;

use glam::{Vec2, Vec3};

use self::debug::DebugThings;
use self::player::Player;
use crate::world::World;

pub mod player;

mod debug;

/// The amount of time that must have passed before the world cleans up its unused data.
///
/// This is done to avoid cleaning up the data too often, which would be a waste of resources
/// and of time (as freeing memory may be relatively expensive in some cases).
const WORLD_CLEAN_UP_INTERVAL: Duration = Duration::from_secs(5);

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
}

impl Game {
    /// Creates a new [`Game`] with the provided seed.
    pub fn new(gpu: Arc<Gpu>, seed: u64) -> Self {
        bns_log::info!("creating a new world with seed: {seed}");
        let generator = Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed));
        let world = World::new(gpu.clone(), generator);
        let player = Player::new(Vec3::new(0.0, 16.0, 0.0));
        let debug = DebugThings::new(gpu.clone());

        Self {
            gpu,
            player,
            world,
            since_last_cleanup: Duration::ZERO,
            seed,
            debug,
        }
    }

    /// Advances the [`Game`] state by one tick.
    #[profiling::function]
    pub fn tick(&mut self, ctx: &mut Ctx) {
        self.debug.reset_overlay();

        if ctx.just_pressed(KeyCode::KeyR) {
            let seed = bns_rng::entropy();
            bns_log::info!("re-creating world with seed: {seed}");
            let generator = Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(self.seed));
            self.world = World::new(self.gpu.clone(), generator);
            self.seed = seed;
        }

        self.since_last_cleanup += ctx.since_last_tick();
        if self.since_last_cleanup >= WORLD_CLEAN_UP_INTERVAL {
            self.world.request_cleanup(
                bns_core::utility::chunk_of(self.player.position()),
                self.player.render_distance() as u32 + 3,
                self.player.vertical_render_distance() as u32 + 3,
            );
            self.since_last_cleanup = Duration::ZERO;
        }

        self.player.tick(ctx);
        self.player.compute_chunks_in_view();
        for &chunk in self.player.chunks_in_view() {
            self.world
                .request_chunk(chunk, -chunk.distance_squared(self.player.position_chunk()));
        }

        // On wasm, no worker threads can be spawned because GPU resources can only be accessed
        // from the main thread apparently (not Send).
        // In that platform, we call `fetch_available_chunks` 10 times to make sure that at
        // least that many chunks are loaded per frame.
        #[cfg(target_arch = "wasm32")]
        {
            for _ in 0..10 {
                self.world.fetch_available_chunks();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.world.fetch_available_chunks();

        let _ = writeln!(self.debug.overlay_buffer(),);

        let _ = writeln!(
            self.debug.overlay_buffer(),
            "Position: {:.2} {:.2} {:.2}\n\
            Chunk: {} {} {}\n\
            Pitch: {:.2}\n\
            Yaw: {:.2}\n\
            \n\
            Loading chunks: {}\n\
            Loaded chunks: {}\n\
            Visible chunks: {}\n\
            \n\
            Seed: {}\n",
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
            self.seed,
        );

        self.debug.tick(ctx);
    }

    /// Renders the game.
    #[profiling::function]
    pub fn render<'res>(&'res mut self, ctx: &mut Ctx, frame: &mut RenderData<'res>) {
        let projection = self.player.camera().projection.matrix();
        let view = self.player.camera().view.matrix(self.player.position());
        frame.uniforms = FrameUniforms {
            inverse_projection: projection.inverse(),
            inverse_view: view.inverse(),
            view,
            projection,
            fog_distance: self.player.render_distance() as f32 * 0.5,
            fog_factor: 1.0 / (self.player.render_distance() as f32 * 6.0),
            resolution: Vec2::new(ctx.width() as f32, ctx.height() as f32),
        };

        let mut total_quad_count = 0;
        for &chunk_pos in self.player.chunks_in_view() {
            let Some(chunk) = self.world.get_existing_chunk(chunk_pos) else {
                continue;
            };

            if chunk.geometry.is_empty() {
                continue;
            }

            let chunk_idx = frame.quads.register_chunk(&ChunkUniforms {
                position: chunk_pos,
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
