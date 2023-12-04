//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;
use std::time::Duration;

use bns_core::{Chunk, ChunkPos};
use bns_render::data::{
    CharacterFlags, CharacterInstance, ChunkUniforms, Color, FrameUniforms, LineInstance,
    LineVertexFlags, RenderData, Ui,
};
use bns_render::{DynamicVertexBuffer, Renderer, RendererConfig, Surface};
use bns_rng::{DefaultRng, FromRng};
use bns_worldgen_std::StandardWorldGenerator;

use glam::{IVec3, Vec2, Vec3};

use winit::event::{KeyEvent, MouseButton};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

use self::camera::Camera;
use crate::window::UserEvent;
use crate::world::World;

mod asset;
mod camera;

const VERTICAL_RENDER_DISTANCE: i32 = 6;

/// The context passed to the functions of [`App`] to control the event loop.
type Ctx = EventLoopWindowTarget<UserEvent>;

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

/// Contains the state of the application.
pub struct App {
    /// A handle to the window that has been opened for the application.
    ///
    /// This can be used to control it.
    window: Arc<Window>,
    /// The surface on which things are rendered.
    surface: Surface<'static>,
    /// The renderer contains the resources required to render things using GPU resources.
    renderer: Renderer,
    /// Some storage to efficiently create [`RenderData`] instances.
    render_data: Option<RenderData<'static>>,

    /// The current state of the camera.
    camera: Camera,

    /// The world that contains all the chunks.
    world: World,

    /// The distance (in chunks) at which chunks are rendered.
    render_distance: i32,

    /// The current state of the chunk debug display.
    debug_chunk_state: DebugChunkState,

    /// The next tick at which the world should be cleaned up.
    next_cleanup: usize,

    /// Whether debug information should be displayed.
    show_debug_info: bool,

    /// The buffer used to build the text to render before sending it to the GPU.
    debug_info_buffer: DynamicVertexBuffer<CharacterInstance>,

    /// The cumulative frame time since the last time `last_frame_time` was computed.
    cumulative_frame_time: Duration,
    /// The number of frames that have been rendered since the last time `last_frame_time` was
    /// computed.
    frames: usize,
    /// The last frame time computed from the frame time history.
    last_frame_time: Duration,

    /// The list of all chunks currently visible from the point of the of the camera.
    chunks_in_view: Vec<ChunkPos>,

    /// The seed that was used to generate the current world.
    current_seed: u64,
}

impl App {
    /// The next tick at which the world should be cleaned up.
    pub const CLEANUP_PERIOD: usize = 200;
    /// The number of frame times to keep in memory.
    pub const FRAME_TIME_HISTORY_SIZE: usize = 10;

    /// Creates a new [`App`] instance.
    ///
    /// # Remarks
    ///
    /// This function must be polled by the web runtime in order to work properly (this obviously
    /// only applied on web).
    pub async fn new(window: Arc<Window>) -> Self {
        let surface = Surface::new(window.clone()).await;
        let renderer = Renderer::new(
            surface.gpu().clone(),
            RendererConfig {
                output_format: surface.info().format,
                texture_atlas: asset::load_texture_atlas().await,
            },
        );
        let seed = bns_rng::entropy();
        let world = World::new(
            renderer.gpu().clone(),
            Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
        );

        const INITIAL_RENDER_DISTANCE: i32 = 16;
        let camera = Camera::new(
            Vec3::new(0.0, 32.0, 0.0),
            render_distance_to_far(INITIAL_RENDER_DISTANCE),
        );

        Self {
            render_data: Some(RenderData::new(renderer.gpu())),
            window,
            surface,
            camera,
            world,
            render_distance: INITIAL_RENDER_DISTANCE,
            debug_chunk_state: DebugChunkState::Hidden,
            next_cleanup: Self::CLEANUP_PERIOD,
            debug_info_buffer: DynamicVertexBuffer::new(renderer.gpu().clone(), 16),
            cumulative_frame_time: Duration::ZERO,
            frames: 0,
            show_debug_info: false,
            renderer,
            last_frame_time: Duration::ZERO,
            chunks_in_view: Vec::new(),
            current_seed: seed,
        }
    }

    /// Notifies the application that the window has been requested to close.
    pub fn notify_close_requested(&mut self, target: &Ctx) {
        target.exit();
    }

    /// Notifies the application that the size of the window on which it is drawing stuff has
    /// changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            // We're probably minimized.
            return;
        }

        self.surface.config_mut().width = width;
        self.surface.config_mut().height = height;
        self.renderer.resize(width, height);
        self.camera.notify_resized(width, height);
    }

    /// Notifies the application that a keyboard event has been received.
    pub fn notify_keyboard(&mut self, _target: &Ctx, event: &KeyEvent) {
        // Toggle fullscreen with F11.
        if event.state.is_pressed() && event.physical_key == KeyCode::F11 {
            self.window.set_fullscreen(
                self.window
                    .fullscreen()
                    .is_none()
                    .then_some(Fullscreen::Borderless(None)),
            );
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::ArrowUp {
            self.render_distance += 2;
            self.camera
                .set_far(render_distance_to_far(self.render_distance));
        }

        if event.state.is_pressed()
            && event.physical_key == KeyCode::ArrowDown
            && self.render_distance > 2
        {
            self.render_distance -= 2;
            self.camera
                .set_far(render_distance_to_far(self.render_distance));
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::KeyR {
            let seed = bns_rng::entropy();
            self.world = World::new(
                self.renderer.gpu().clone(),
                Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
            );
            self.current_seed = seed;
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::F10 {
            self.debug_chunk_state = self.debug_chunk_state.next_state();
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::F3 {
            self.show_debug_info = !self.show_debug_info;
        }

        self.camera.notify_keyboard(event);
    }

    /// Notifies the application that a mouse input event has been received.
    pub fn notify_mouse_input(
        &mut self,
        _target: &Ctx,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        if state.is_pressed() && button == MouseButton::Left {
            self.window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
                .expect("failed to grab the mouse cursor");
            self.window.set_cursor_visible(false);
        }
    }

    /// Notifies the application that the window has gained or lost focus.
    pub fn notify_focused(&mut self, _target: &Ctx, now_focused: bool) {
        if !now_focused {
            self.window
                .set_cursor_grab(CursorGrabMode::None)
                .expect("failed to release the cursor grab");
            self.window.set_cursor_visible(true);
        }
    }

    /// Notifies the application that the mouse has moved.
    ///
    /// Note that the provided coordinates are not in pixels, but instead are an arbitrary
    /// value relative to the last reported mouse position.
    pub fn notify_mouse_moved(&mut self, _target: &Ctx, dx: f64, dy: f64) {
        self.camera.notify_mouse_moved(dx, dy);
    }

    /// Renders a frame to the window.
    #[profiling::function]
    pub fn render(&mut self) {
        let Some(frame) = self.surface.acquire_image() else {
            return;
        };

        let mut render_data = self.render_data.take().unwrap();

        let view = self.camera.view_matrix();
        let projection = self.camera.projection_matrix();
        render_data.frame = FrameUniforms {
            inverse_projection: projection.inverse(),
            projection,
            inverse_view: view.inverse(),
            view,
            resolution: Vec2::new(
                self.surface.config().width as f32,
                self.surface.config().height as f32,
            ),
            fog_factor: 1.0 / (self.render_distance as f32 * 6.0),
            fog_distance: self.render_distance as f32 * 0.5,
        };
        let mut total_quad_count = 0;
        for &chunk_pos in &self.chunks_in_view {
            if let Some(chunk) = self.world.get_existing_chunk(chunk_pos) {
                if chunk.geometry.is_empty() {
                    continue;
                }

                let chunk_idx = render_data.quads.register_chunk(&ChunkUniforms {
                    position: chunk_pos,
                });

                if let Some(buffer) = chunk.geometry.opaque_quad_instances() {
                    render_data
                        .quads
                        .register_opaque_quads(chunk_idx, buffer.slice());
                    total_quad_count += buffer.len();
                }
                if let Some(buffer) = chunk.geometry.transparent_quad_instances() {
                    render_data
                        .quads
                        .register_transparent_quads(chunk_idx, buffer.slice());
                    total_quad_count += buffer.len();
                }
            }
        }

        const CURRENT_CHUNK_COLOR: Color = Color::RED;
        const OTHER_CHUNK_COLOR: Color = Color::YELLOW;
        match self.debug_chunk_state {
            DebugChunkState::Hidden => (),
            DebugChunkState::ShowAllChunks => {
                const S: f32 = Chunk::SIDE as f32;
                let chunk_pos = chunk_of(self.camera.position()).as_vec3() * S;
                let count = self.render_distance.min(6);
                let bound = count as f32 * S;
                for a in -count..=count {
                    let a = a as f32 * S;
                    for b in -count..=count {
                        let b = b as f32 * S;

                        render_data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(bound, a, b),
                            end: chunk_pos + Vec3::new(-bound, a, b),
                            color: OTHER_CHUNK_COLOR,
                            width: 1.0,
                            flags: LineVertexFlags::empty(),
                        });
                        render_data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(a, bound, b),
                            end: chunk_pos + Vec3::new(a, -bound, b),
                            color: OTHER_CHUNK_COLOR,
                            flags: LineVertexFlags::empty(),
                            width: 1.0,
                        });
                        render_data.lines.push(LineInstance {
                            start: chunk_pos + Vec3::new(a, b, bound),
                            end: chunk_pos + Vec3::new(a, b, -bound),
                            color: OTHER_CHUNK_COLOR,
                            flags: LineVertexFlags::empty(),
                            width: 1.0,
                        });
                    }
                }

                add_aabb_lines(
                    &mut render_data,
                    chunk_pos,
                    chunk_pos + Vec3::splat(S),
                    CURRENT_CHUNK_COLOR,
                    3.0,
                    LineVertexFlags::ABOVE,
                );
            }
            DebugChunkState::ShowCurrentChunk => {
                const S: f32 = Chunk::SIDE as f32;

                let chunk_pos = chunk_of(self.camera.position()).as_vec3() * S;
                add_aabb_lines(
                    &mut render_data,
                    chunk_pos,
                    chunk_pos + Vec3::splat(S),
                    CURRENT_CHUNK_COLOR,
                    2.0,
                    LineVertexFlags::ABOVE,
                );
            }
        }

        if self.show_debug_info {
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
                frame_time = self.last_frame_time,
                fps = 1.0 / self.last_frame_time.as_secs_f64(),
                x = self.camera.position().x,
                y = self.camera.position().y,
                z = self.camera.position().z,
                pitch = self.camera.pitch().to_degrees(),
                yaw = self.camera.yaw().to_degrees(),
                render_distance = self.render_distance,
                loading_chunks = self.world.loading_chunk_count(),
                loaded_chunks = self.world.loaded_chunk_count(),
                visible_chunks = self.chunks_in_view.len(),
                seed = self.current_seed,
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

            self.debug_info_buffer.clear();
            self.debug_info_buffer.extend(&buf);

            render_data
                .ui
                .push(Ui::Text(self.debug_info_buffer.slice()));
        }

        self.renderer.render(frame.target(), &mut render_data);
        frame.present();

        self.render_data = Some(render_data.reset());
    }

    /// Advances the state of the application by one tick.
    #[profiling::function]
    pub fn tick(&mut self, _target: &Ctx, dt: Duration) {
        let delta_seconds = dt.as_secs_f32();
        self.cumulative_frame_time += dt;
        self.frames += 1;
        if self.frames == Self::FRAME_TIME_HISTORY_SIZE {
            self.last_frame_time =
                self.cumulative_frame_time / Self::FRAME_TIME_HISTORY_SIZE as u32;
            self.frames = 0;
            self.cumulative_frame_time = Duration::ZERO;
        }

        self.next_cleanup -= 1;
        if self.next_cleanup == 0 {
            self.world.request_cleanup(
                chunk_of(self.camera.position()),
                self.render_distance as u32 + 3,
                VERTICAL_RENDER_DISTANCE as u32 + 3,
            );
            self.next_cleanup = Self::CLEANUP_PERIOD;
        }

        self.camera.tick(delta_seconds);

        self.chunks_in_view.clear();
        chunks_in_frustum(&self.camera, self.render_distance, &mut self.chunks_in_view);

        let center = chunk_of(self.camera.position());
        for &chunk in &self.chunks_in_view {
            self.world
                .request_chunk(chunk, -chunk.distance_squared(center));
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
    }
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

    // OPTIMZE: make sure that the vector is directly written to memory and not copied
    // from stack.

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
