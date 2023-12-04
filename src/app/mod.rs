//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;
use std::time::Duration;

use bns_app::{App, KeyCode, MouseButton};
use bns_core::{Chunk, ChunkPos};
use bns_render::data::{
    CharacterFlags, CharacterInstance, ChunkUniforms, Color, FrameUniforms, LineInstance,
    LineVertexFlags, RenderData, Ui,
};
use bns_render::{DynamicVertexBuffer, Renderer, RendererConfig, Surface};
use bns_rng::{DefaultRng, FromRng};
use bns_worldgen_std::StandardWorldGenerator;

use glam::{IVec3, Vec2, Vec3};

use self::camera::Camera;
use crate::world::World;

mod asset;
mod camera;

/// The state of the debug chunk display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugChunkState {
    /// No debug information are displayed.
    Hidden,
    /// Draw the chunk that the camera is currently in.
    ShowCurrentChunk,
    /// Draw the chunk grid.
    ShowAllChunks,
}

impl DebugChunkState {
    /// Returns the next state in the cycle.
    #[inline]
    pub fn next_state(self) -> Self {
        match self {
            Self::Hidden => Self::ShowCurrentChunk,
            Self::ShowCurrentChunk => Self::ShowAllChunks,
            Self::ShowAllChunks => Self::Hidden,
        }
    }
}

/// The number of ticks between two world cleanups.
const CLEANUP_PERIOD: Duration = Duration::from_secs(5);
/// The amount of time required to compute the average frame time.
const CUMULATIVE_FRAME_TIME_THRESHOLD: Duration = Duration::from_secs(1);
/// The maximum and minimum render distance that can be set (in chunks).
const RENDER_DISTANCE_RANGE: (i32, i32) = (2, 32);
/// The initial render distance.
const INITIAL_RENDER_DISTNACE: i32 = 8;
/// The initial position of the player.
const INITIAL_PLAYER_POS: Vec3 = Vec3::new(0.0, 16.0, 0.0);
/// The render distance on the Y axis.
const VERTICAL_RENDER_DISTANCE: i32 = 6;

/// Runs the application until completion.
pub fn run() {
    // On web, we need everything to be executed by the browser's executor (because of some
    // internals of WebGPU).
    //
    // Depending on the target platform, the `run_async` function will either be executed
    // by the browser's executor, or by a dummy runtime.
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(run_async());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run_async());
    }
}

async fn run_async() {
    let app = App::new(bns_app::Config {
        title: "Blocks 'n Stuff",
        min_size: (300, 300),
        fullscreen: cfg!(not(debug_assertions)),
    });

    let mut surface = Surface::new(app.opaque_window()).await;
    let mut renderer = Renderer::new(
        surface.gpu().clone(),
        RendererConfig {
            output_format: surface.info().format,
            texture_atlas: asset::load_texture_atlas().await,
        },
    );
    let mut render_data = Some(RenderData::new(surface.gpu()));

    let mut seed = bns_rng::entropy();
    let mut world = World::new(
        renderer.gpu().clone(),
        Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
    );
    let mut since_last_cleanup = Duration::ZERO;
    let mut chunks_in_view = Vec::new();

    let mut render_distance = INITIAL_RENDER_DISTNACE;

    let mut camera = Camera::new(INITIAL_PLAYER_POS, render_distance_to_far(render_distance));

    let mut chunk_debug_state = DebugChunkState::Hidden;
    let mut debug_overlay = false;
    let mut cumulative_frame_time = Duration::ZERO;
    let mut accumulated_frames = 0;
    let mut debug_frame_time = Duration::ZERO;
    let mut debug_info_buffer = DynamicVertexBuffer::new(surface.gpu().clone(), 64);

    app.run(|ctx| {
        // ==============================================
        // Misc Events
        // ==============================================

        if ctx.just_resized() {
            surface.config_mut().width = ctx.width();
            surface.config_mut().height = ctx.height();
            renderer.resize(ctx.width(), ctx.height());
        }

        // ==============================================
        // Input Handling
        // ==============================================

        #[cfg(not(target_arch = "wasm32"))]
        {
            if ctx.just_pressed(KeyCode::Escape) {
                ctx.close();
                return;
            }
        }

        if ctx.just_pressed(KeyCode::F11) {
            ctx.set_fullscreen(!ctx.fullscreen());
        }

        if ctx.just_pressed(KeyCode::ArrowUp) {
            render_distance += 2;
            if render_distance >= RENDER_DISTANCE_RANGE.1 {
                render_distance = RENDER_DISTANCE_RANGE.1;
            }
            camera.set_far(render_distance_to_far(render_distance));
        }

        if ctx.just_pressed(KeyCode::ArrowDown) {
            render_distance -= 2;
            if render_distance <= RENDER_DISTANCE_RANGE.0 {
                render_distance = RENDER_DISTANCE_RANGE.0;
            }
            camera.set_far(render_distance_to_far(render_distance));
        }

        if ctx.just_pressed(KeyCode::KeyR) {
            seed = bns_rng::entropy();
            world = World::new(
                renderer.gpu().clone(),
                Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
            );
        }

        if ctx.just_pressed(KeyCode::F10) {
            chunk_debug_state = chunk_debug_state.next_state();
        }

        if ctx.just_pressed(KeyCode::F3) {
            debug_overlay = !debug_overlay;
        }

        if ctx.just_pressed(MouseButton::Left) {
            ctx.grab_cursor();
        }

        // ==============================================
        // Tick other objects
        // ==============================================

        cumulative_frame_time += ctx.since_last_tick();
        accumulated_frames += 1;
        if cumulative_frame_time >= CUMULATIVE_FRAME_TIME_THRESHOLD {
            debug_frame_time = cumulative_frame_time / accumulated_frames;
            cumulative_frame_time = Duration::ZERO;
            accumulated_frames = 0;
        }

        since_last_cleanup += ctx.since_last_tick();
        if since_last_cleanup >= CLEANUP_PERIOD {
            world.request_cleanup(
                chunk_of(camera.position()),
                render_distance as u32 + 3,
                VERTICAL_RENDER_DISTANCE as u32 + 3,
            );
            since_last_cleanup = Duration::ZERO;
        }

        camera.tick(ctx);

        chunks_in_view.clear();
        chunks_in_frustum(&camera, render_distance, &mut chunks_in_view);

        let center = chunk_of(camera.position());
        for &chunk in &chunks_in_view {
            world.request_chunk(chunk, -chunk.distance_squared(center));
        }

        // On wasm, no worker threads can be spawned because GPU resources can only be accessed
        // from the main thread apparently (not Send).
        // In that platform, we call `fetch_available_chunks` 10 times to make sure that at
        // least that many chunks are loaded per frame.
        #[cfg(target_arch = "wasm32")]
        {
            for _ in 0..10 {
                world.fetch_available_chunks();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        world.fetch_available_chunks();

        // ==============================================
        // Rendering
        // ==============================================

        let Some(frame) = surface.acquire_image() else {
            return;
        };

        let mut data = render_data.take().unwrap();

        let view = camera.view_matrix();
        let projection = camera.projection_matrix();
        data.frame = FrameUniforms {
            inverse_projection: projection.inverse(),
            projection,
            inverse_view: view.inverse(),
            view,
            resolution: Vec2::new(ctx.width() as f32, ctx.height() as f32),
            fog_factor: 1.0 / (render_distance as f32 * 6.0),
            fog_distance: render_distance as f32 * 0.5,
        };
        let mut total_quad_count = 0;
        for &chunk_pos in &chunks_in_view {
            if let Some(chunk) = world.get_existing_chunk(chunk_pos) {
                if chunk.geometry.is_empty() {
                    continue;
                }

                let chunk_idx = data.quads.register_chunk(&ChunkUniforms {
                    position: chunk_pos,
                });

                if let Some(buffer) = chunk.geometry.opaque_quad_instances() {
                    data.quads.register_opaque_quads(chunk_idx, buffer.slice());
                    total_quad_count += buffer.len();
                }
                if let Some(buffer) = chunk.geometry.transparent_quad_instances() {
                    data.quads
                        .register_transparent_quads(chunk_idx, buffer.slice());
                    total_quad_count += buffer.len();
                }
            }
        }

        const CURRENT_CHUNK_COLOR: Color = Color::RED;
        const OTHER_CHUNK_COLOR: Color = Color::YELLOW;
        match chunk_debug_state {
            DebugChunkState::Hidden => (),
            DebugChunkState::ShowAllChunks => {
                const S: f32 = Chunk::SIDE as f32;
                let chunk_pos = chunk_of(camera.position()).as_vec3() * S;
                let count = render_distance.min(6);
                let bound = count as f32 * S;
                for a in -count..=count {
                    let a = a as f32 * S;
                    for b in -count..=count {
                        let b = b as f32 * S;

                        data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(bound, a, b),
                            end: chunk_pos + Vec3::new(-bound, a, b),
                            color: OTHER_CHUNK_COLOR,
                            width: 1.0,
                            flags: LineVertexFlags::empty(),
                        });
                        data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(a, bound, b),
                            end: chunk_pos + Vec3::new(a, -bound, b),
                            color: OTHER_CHUNK_COLOR,
                            flags: LineVertexFlags::empty(),
                            width: 1.0,
                        });
                        data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(a, b, bound),
                            end: chunk_pos + Vec3::new(a, b, -bound),
                            color: OTHER_CHUNK_COLOR,
                            flags: LineVertexFlags::empty(),
                            width: 1.0,
                        });
                    }
                }

                add_aabb_lines(
                    &mut data,
                    chunk_pos,
                    chunk_pos + Vec3::splat(S),
                    CURRENT_CHUNK_COLOR,
                    3.0,
                    LineVertexFlags::ABOVE,
                );
            }
            DebugChunkState::ShowCurrentChunk => {
                const S: f32 = Chunk::SIDE as f32;

                let chunk_pos = chunk_of(camera.position()).as_vec3() * S;
                add_aabb_lines(
                    &mut data,
                    chunk_pos,
                    chunk_pos + Vec3::splat(S),
                    CURRENT_CHUNK_COLOR,
                    2.0,
                    LineVertexFlags::ABOVE,
                );
            }
        }

        if debug_overlay {
            let s = format!(
                "\
                Frame time: {frame_time:?} ({fps:.2} fps)\n\
                \n\
                Position: {x:.2} {y:.2} {z:.2}\n\
                Pitch: {pitch}\n\
                Yaw: {yaw}\n\
                \n\
                Render distance: {render_distance}\n\
                Total quads: {total_quads}\n\
                \n\
                Loading chunks: {loading_chunks}\n\
                Loaded chunks: {loaded_chunks}\n\
                Visible chunks: {visible_chunks}\n\
                \n\
                Seed: {seed}\n\
                ",
                frame_time = ctx.since_last_tick(),
                fps = 1.0 / ctx.since_last_tick().as_secs_f64(),
                x = camera.position().x,
                y = camera.position().y,
                z = camera.position().z,
                pitch = camera.pitch().to_degrees(),
                yaw = camera.yaw().to_degrees(),
                render_distance = render_distance,
                loading_chunks = world.loading_chunk_count(),
                loaded_chunks = world.loaded_chunk_count(),
                visible_chunks = chunks_in_view.len(),
                seed = seed,
                total_quads = total_quad_count,
            );
            let mut buf = Vec::new();
            build_text(
                &mut buf,
                &s,
                Color::WHITE,
                Vec2::new(10.0, 10.0),
                Vec2::new(0.0, 4.0),
                Vec2::new(16.0, 32.0),
            );

            debug_info_buffer.clear();
            debug_info_buffer.extend(&buf);

            data.ui.push(Ui::Text(debug_info_buffer.slice()));
        }

        renderer.render(frame.target(), &mut data);
        frame.present();

        render_data = Some(data.reset());
    });
}

/// Converts a render distance measured in chunks to a far plane for the camera.
fn render_distance_to_far(render_distance: i32) -> f32 {
    (render_distance as f32 + 2.0) * Chunk::SIDE as f32
}

/// Calls the provided function for every visible chunk from the camera.
fn chunks_in_frustum(camera: &Camera, render_distance: i32, buf: &mut Vec<ChunkPos>) {
    const CHUNK_RADIUS: f32 = (Chunk::SIDE as f32) * 0.8660254; // sqrt(3) / 2

    let camera_chunk_pos = chunk_of(camera.position());

    for x in -render_distance..=render_distance {
        for y in -VERTICAL_RENDER_DISTANCE..=VERTICAL_RENDER_DISTANCE {
            for z in -render_distance..=render_distance {
                if x * x + z * z > render_distance * render_distance {
                    continue;
                }

                let relative_chunk_pos = IVec3::new(x, y, z);
                let relative_chunk_pos_center = (relative_chunk_pos.as_vec3() + Vec3::splat(0.5))
                    * Chunk::SIDE as f32
                    - (camera.position() - camera_chunk_pos.as_vec3() * Chunk::SIDE as f32);

                if camera.is_sphere_in_frustum(relative_chunk_pos_center, CHUNK_RADIUS) {
                    buf.push(camera_chunk_pos + relative_chunk_pos);
                }
            }
        }
    }
}

/// Returns the chunk that the provided position is in.
fn chunk_of(pos: Vec3) -> ChunkPos {
    fn coord_to_chunk(coord: f32) -> i32 {
        if coord >= 0.0 {
            coord as i32 / Chunk::SIDE
        } else {
            coord as i32 / Chunk::SIDE - 1
        }
    }

    ChunkPos::new(
        coord_to_chunk(pos.x),
        coord_to_chunk(pos.y),
        coord_to_chunk(pos.z),
    )
}

/// Adds a new axis-aligned bounding box to the gizmos list.
pub fn add_aabb_lines(
    render_data: &mut RenderData,
    min: Vec3,
    max: Vec3,
    color: Color,
    width: f32,
    flags: LineVertexFlags,
) {
    use glam::vec3;

    let base = LineInstance {
        width,
        flags,
        color,
        start: Vec3::ZERO,
        end: Vec3::ZERO,
    };

    render_data.lines.extend_from_slice(&[
        // Lower face
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(max.x, min.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(min.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, min.y, min.z),
            ..base
        },
        // Upper face
        LineInstance {
            start: vec3(min.x, max.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, min.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, max.y, max.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        // Vertical edges
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
    ]);
}

fn build_text(
    buf: &mut Vec<CharacterInstance>,
    text: &str,
    color: Color,
    mut position: Vec2,
    space: Vec2,
    size: Vec2,
) {
    let initial_x = position.x;

    for text in text.chars() {
        if text == '\n' {
            position.x = initial_x;
            position.y += size.y + space.y;
            continue;
        }

        let flags = CharacterFlags::from_character(text)
            .or(CharacterFlags::from_character(' '))
            .unwrap();
        buf.push(CharacterInstance {
            flags,
            color,
            position,
            size,
        });
        position.x += size.x + space.x;
    }
}
