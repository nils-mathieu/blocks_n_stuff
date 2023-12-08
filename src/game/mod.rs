//! Contains the state of the game world (not including eventual menus).

use std::sync::Arc;
use std::time::Duration;

use bns_app::{Ctx, KeyCode};
use bns_core::ChunkPos;
use bns_render::data::{ChunkUniforms, Color, FrameFlags, FrameUniforms, LineFlags, RenderData};
use bns_render::Gpu;
use bns_rng::{DefaultRng, FromRng, Rng};
use bns_worldgen_std::StandardWorldGenerator;

use glam::{Mat4, Quat, Vec2, Vec3};
use rodio::Source;

use self::debug::DebugThings;
use self::player::{LookingAt, Player};
use crate::assets::{Assets, Sounds};
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

    /// The handle to the output stream that's used to play the music.
    stream_handle: rodio::OutputStreamHandle,
    /// The stream that's used to play the music.
    ///
    /// This must not be dropped until we don't need to play music anymore.
    _stream: rodio::OutputStream,

    /// The random number generator for the game.
    rng: DefaultRng,

    /// The current in-game time, used to determine the position of the sun.
    time: Duration,
}

impl Game {
    /// Creates a new [`Game`] with the provided seed.
    pub fn new(gpu: Arc<Gpu>, sounds: &Sounds) -> Self {
        let seed = bns_rng::entropy();

        bns_log::info!("creating a new world with seed: {seed}");
        let generator = Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed));
        let world = World::new(gpu.clone(), generator);
        let player = Player::new(gpu.clone(), Vec3::new(0.0, 16.0, 0.0));
        let debug = DebugThings::new(gpu.clone());

        let (_stream, stream_handle) =
            rodio::OutputStream::try_default().expect("failed to find an audio device");

        // Play the background music in a loop.
        stream_handle
            .play_raw(
                rodio::Decoder::new_vorbis(std::io::Cursor::new(sounds.background_music.clone()))
                    .unwrap()
                    .repeat_infinite()
                    .convert_samples(),
            )
            .unwrap();

        Self {
            gpu,
            player,
            world,
            since_last_cleanup: Duration::ZERO,
            seed,
            debug,
            fog_enabled: true,

            stream_handle,
            _stream,

            rng: DefaultRng::from_entropy(),

            time: Duration::ZERO,
        }
    }

    /// Advances the [`Game`] state by one tick.
    #[profiling::function]
    pub fn tick(&mut self, ctx: &mut Ctx, sounds: &Sounds) {
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

        self.player.tick(
            &mut self.world,
            &self.stream_handle,
            sounds,
            &mut self.rng,
            ctx,
        );
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
            Pitch: {:.2}, Yaw: {:.2} (toward {})\n\
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
            DisplayTowards(self.player.camera().view.yaw()),
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

        // Compute the direction of the sun.
        let sub_day = (self.time.as_millis() % 600000) as f32 / 600000.0;
        let sun_direction = Quat::from_rotation_y(sub_day * std::f32::consts::TAU)
            * Vec3::new(0.0, 1.0, -1.5).normalize();

        // Initialize the frame.
        let projection = self.player.camera().projection.matrix();
        let view = self
            .player
            .camera()
            .view
            .matrix(self.player.head_position());
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
            flags: if self.player.is_underwater() {
                FrameFlags::UNDERWATER
            } else {
                FrameFlags::empty()
            },
            milliseconds: ctx.since_startup().as_millis() as u32,
            sun_direction,
            fog_height: if self.player.is_underwater() {
                0.2
            } else {
                2.0
            },
            light_transform: Mat4::orthographic_lh(-50.0, 50.0, -50.0, 50.0, 1.0, 100.0)
                * Mat4::look_to_lh(
                    self.player.position() + sun_direction * 50.0,
                    -sun_direction,
                    Vec3::Y,
                ),
        };
        frame.fog_enabled = self.fog_enabled;

        if ctx.pressing(KeyCode::KeyU) {
            self.time += ctx.since_last_tick() * 50;
        } else {
            self.time += ctx.since_last_tick();
        }

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
            const PADDING: f32 = 0.0;

            utility::push_aabb_lines(
                &mut frame.lines,
                looking_at.world_pos.as_vec3() - Vec3::ONE * PADDING,
                looking_at.world_pos.as_vec3() + Vec3::ONE * (1.0 + PADDING),
                Color::WHITE,
                2.0,
                LineFlags::WITH_BIAS,
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

/// A simple wrapper that implement [`std::fmt::Display`] to display the direction
/// that the player is currently looking at (given its YAW value).
struct DisplayTowards(f32);

impl std::fmt::Display for DisplayTowards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let yaw = self.0;

        if yaw < 45f32.to_radians() {
            write!(f, "+Z")
        } else if yaw < 135f32.to_radians() {
            write!(f, "+X")
        } else if yaw < 225f32.to_radians() {
            write!(f, "-Z")
        } else {
            write!(f, "-X")
        }
    }
}
