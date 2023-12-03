//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;

use bns_core::{Chunk, ChunkPos, TextureId};
use bns_render::data::{
    ChunkUniforms, Color, FrameUniforms, LineInstance, LineVertexFlags, RenderData,
};
use bns_render::{Renderer, RendererConfig, Surface, TextureAtlasConfig, TextureFormat};
use bns_rng::{DefaultRng, FromRng};
use bns_workers::Priority;
use bns_worldgen_std::StandardWorldGenerator;

use glam::{IVec3, Vec2, Vec3};

use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

use crate::window::UserEvent;
use crate::world::World;

mod camera;

use self::camera::Camera;

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
}

impl App {
    /// The next tick at which the world should be cleaned up.
    pub const CLEANUP_PERIOD: usize = 200;

    /// Creates a new [`App`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let surface = Surface::new(window.clone());
        let renderer = Renderer::new(
            surface.gpu().clone(),
            RendererConfig {
                output_format: surface.info().format,
                texture_atlas: load_texture_atlas(),
            },
        );
        let seed = bns_rng::entropy();
        println!("Seed: {seed}");
        let world = World::new(
            renderer.gpu().clone(),
            Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
        );

        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .expect("failed to grab the mouse cursor");
        window.set_cursor_visible(false);

        const INITIAL_RENDER_DISTANCE: i32 = 16;
        let camera = Camera::new(
            Vec3::new(0.0, 32.0, 0.0),
            render_distance_to_far(INITIAL_RENDER_DISTANCE),
        );

        Self {
            render_data: Some(RenderData::new(renderer.gpu())),
            window,
            surface,
            renderer,
            camera,
            world,
            render_distance: INITIAL_RENDER_DISTANCE,
            debug_chunk_state: DebugChunkState::Hidden,
            next_cleanup: Self::CLEANUP_PERIOD,
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
    pub fn notify_keyboard(&mut self, target: &Ctx, event: &KeyEvent) {
        // TODO: remove this when a menu is implemented to exit the application.
        // The key to open the menu will probably be Escape key anyway so I won't
        // miss this.
        if event.state.is_pressed() && event.physical_key == KeyCode::Escape {
            target.exit();
        }

        // Toggle fullscreen with F11.
        if event.state.is_pressed() && event.physical_key == KeyCode::F11 {
            self.window.set_fullscreen(
                self.window
                    .fullscreen()
                    .is_none()
                    .then_some(Fullscreen::Borderless(None)),
            );
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::KeyI {
            println!("Chunks in flight: {}", self.world.chunks_in_flight());
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::ArrowUp {
            self.render_distance += 2;
            self.camera
                .set_far(render_distance_to_far(self.render_distance));
            println!("Render distance: {}", self.render_distance);
        }

        if event.state.is_pressed()
            && event.physical_key == KeyCode::ArrowDown
            && self.render_distance > 2
        {
            self.render_distance -= 2;
            self.camera
                .set_far(render_distance_to_far(self.render_distance));
            println!("Render distance: {}", self.render_distance);
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::KeyR {
            let seed = bns_rng::entropy();
            println!("Seed: {seed}");
            self.world = World::new(
                self.renderer.gpu().clone(),
                Arc::new(StandardWorldGenerator::from_seed::<DefaultRng>(seed)),
            );
        }

        if event.state.is_pressed() && event.physical_key == KeyCode::F10 {
            self.debug_chunk_state = self.debug_chunk_state.next_state();
            println!("Debug chunk state: {:?}", self.debug_chunk_state);
        }

        self.camera.notify_keyboard(event);
    }

    /// Notifies the application that the mouse has moved.
    ///
    /// Note that the provided coordinates are not in pixels, but instead are an arbitrary
    /// value relative to the last reported mouse position.
    pub fn notify_mouse_moved(&mut self, _target: &Ctx, dx: f64, dy: f64) {
        self.camera.notify_mouse_moved(dx, dy);
    }

    /// Renders a frame to the window.
    pub fn render(&mut self) {
        // TODO: print quad count in debug info.

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
            fog_factor: 1.0 / (self.render_distance as f32 * 12.0),
            _padding: 0,
        };
        chunks_in_frustum(&self.camera, self.render_distance, |chunk_pos, _| {
            if let Some(chunk) = self.world.get_existing_chunk(chunk_pos) {
                if chunk.geometry.is_empty() {
                    return;
                }

                let chunk_idx = render_data.quads.reigster_chunk(&ChunkUniforms {
                    position: chunk_pos,
                });

                if let Some(buffer) = chunk.geometry.opaque_quad_instances() {
                    render_data.quads.register_opaque_quads(chunk_idx, buffer);
                }
                if let Some(buffer) = chunk.geometry.transparent_quad_instances() {
                    render_data
                        .quads
                        .register_transparent_quads(chunk_idx, buffer);
                }
            }
        });

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

        self.renderer.render(frame.target(), &mut render_data);
        frame.present();

        self.render_data = Some(render_data.reset());
    }

    /// Advances the state of the application by one tick.
    pub fn tick(&mut self, _target: &Ctx, dt: f32) {
        self.next_cleanup -= 1;
        if self.next_cleanup == 0 {
            self.world.request_cleanup(
                chunk_of(self.camera.position()),
                self.render_distance as u32 + 3,
                VERTICAL_RENDER_DISTANCE as u32 + 3,
            );
            self.next_cleanup = Self::CLEANUP_PERIOD;
        }

        self.camera.tick(dt);

        chunks_in_frustum(&self.camera, self.render_distance, |chunk_pos, priority| {
            self.world.request_chunk(chunk_pos, priority);
        });

        for _ in 0..10 {
            self.world.fetch_available_chunks();
        }
    }
}

/// Converts a render distance measured in chunks to a far plane for the camera.
fn render_distance_to_far(render_distance: i32) -> f32 {
    (render_distance as f32 + 2.0) * Chunk::SIDE as f32
}

/// Calls the provided function for every visible chunk from the camera.
fn chunks_in_frustum(
    camera: &Camera,
    render_distance: i32,
    mut callback: impl FnMut(ChunkPos, Priority),
) {
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
                    let priority = Priority::MAX - (x * x + y * y + z * z) as Priority;
                    callback(camera_chunk_pos + relative_chunk_pos, priority);
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

fn load_texture_atlas() -> TextureAtlasConfig<'static> {
    let mut data = Vec::new();
    let mut count = 0;
    let mut metadata = None;

    for texture_id in TextureId::all() {
        let path = format!("assets/{}.png", texture_id.file_name());
        let mut image = bns_image::Image::load_png(std::fs::File::open(path).unwrap()).unwrap();
        image.ensure_srgb();
        image.ensure_rgba();

        match &metadata {
            Some(metadata) => assert_eq!(metadata, &image.metadata),
            None => metadata = Some(image.metadata),
        }

        data.extend_from_slice(&image.pixels);
        count += 1;
    }

    let metadata = metadata.unwrap();

    TextureAtlasConfig {
        data: data.into(),
        width: metadata.width,
        height: metadata.height,
        count,
        mip_level_count: 1,
        format: match metadata.color_space {
            bns_image::ColorSpace::Srgb => TextureFormat::Rgba8UnormSrgb,
            bns_image::ColorSpace::Unknown => TextureFormat::Rgba8Unorm,
            bns_image::ColorSpace::Linear => TextureFormat::Rgba8Unorm,
        },
    }
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
