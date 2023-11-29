//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;

use glam::{IVec3, Vec3, Vec4};
use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

use crate::gfx::render_data::{BufferSlice, ChunkUniforms, FrameUniforms, RenderData};
use crate::gfx::Renderer;
use crate::window::UserEvent;
use crate::world::{Chunk, ChunkPos, World};
use crate::worldgen::StandardWorldGenerator;

mod camera;

use self::camera::Camera;

/// The context passed to the functions of [`App`] to control the event loop.
type Ctx = EventLoopWindowTarget<UserEvent>;

/// Contains the state of the application.
pub struct App {
    /// A handle to the window that has been opened for the application.
    ///
    /// This can be used to control it.
    window: Arc<Window>,
    /// The renderer contains the resources required to render things using GPU resources.
    renderer: Renderer,

    /// The current state of the camera.
    camera: Camera,

    /// The world that contains all the chunks.
    world: World,
}

impl App {
    /// Creates a new [`App`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());

        let world = World::new(
            renderer.gpu().clone(),
            Box::new(StandardWorldGenerator::new()),
        );

        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .expect("failed to grab the mouse cursor");
        window.set_cursor_visible(false);

        Self {
            window,
            renderer,
            camera: Camera::default(),
            world,
        }
    }

    /// Notifies the application that the window has been requested to close.
    pub fn notify_close_requested(&mut self, target: &Ctx) {
        target.exit();
    }

    /// Notifies the application that the size of the window on which it is drawing stuff has
    /// changed.
    pub fn notify_resized(&mut self, _target: &Ctx, width: u32, height: u32) {
        self.renderer.notify_resized(width, height);
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

        self.camera.notify_keyboard(event);
    }

    /// Notifies the application that the mouse has moved.
    ///
    /// Note that the provided coordinates are not in pixels, but instead are an arbitrary
    /// value relative to the last reported mouse position.
    pub fn notify_mouse_moved(&mut self, _taget: &Ctx, dx: f64, dy: f64) {
        self.camera.notify_mouse_moved(dx, dy);
    }

    // OPTIMIZE: manging to make this function take a shared reference would probably make the
    // code way cleaner. That would probably require to smartly load chunks in the tick function
    // instead.
    /// Renders a frame to the window.
    pub fn render(&mut self) {
        let mut quad_buffers = Vec::new();
        let mut chunk_uniforms = Vec::new();

        chunks_in_frustum(&self.camera, |chunk_pos| {
            if let Some(chunk) = self.world.get_existing_chunk(chunk_pos) {
                if let Some((count, quads)) = &chunk.geometry.quads {
                    quad_buffers.push(BufferSlice::new(quads, *count));
                    chunk_uniforms.push(ChunkUniforms {
                        position: chunk_pos,
                        _padding: [0; 13],
                    });
                }
            }
        });

        self.renderer.render(&RenderData {
            clear_color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            frame_uniforms: FrameUniforms {
                camera: self.camera.matrix(),
            },
            chunk_uniforms: &chunk_uniforms,
            quads: &quad_buffers,
        });
    }

    /// Advances the state of the application by one tick.
    pub fn tick(&mut self, _target: &Ctx, dt: f32) {
        self.camera.tick(dt);

        chunks_in_frustum(&self.camera, |chunk_pos| {
            self.world.request_chunk(chunk_pos, 0);
        });
    }
}

/// Calls the provided function for every visible chunk from the camera.
fn chunks_in_frustum(camera: &Camera, mut callback: impl FnMut(ChunkPos)) {
    const HORIZONTAL_RENDER_DISTANCE: i32 = 8;
    const VERTICAL_RENDER_DISTANCE: i32 = 8;
    const CHUNK_RADIUS: f32 = (Chunk::SIDE as f32) * 0.8660254; // sqrt(3) / 2

    fn coord_to_chunk(coord: f32) -> i32 {
        if coord >= 0.0 {
            coord as i32 / Chunk::SIDE
        } else {
            (coord as i32 - Chunk::SIDE + 1) / Chunk::SIDE
        }
    }

    let camera_chunk_pos = ChunkPos::new(
        coord_to_chunk(camera.position().x),
        coord_to_chunk(camera.position().y),
        coord_to_chunk(camera.position().z),
    );
    for x in -HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE {
        for y in -VERTICAL_RENDER_DISTANCE..=VERTICAL_RENDER_DISTANCE {
            for z in -HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE {
                if x * x + z * z > HORIZONTAL_RENDER_DISTANCE * HORIZONTAL_RENDER_DISTANCE {
                    continue;
                }

                let relative_chunk_pos = IVec3::new(x, y, z);
                let relative_chunk_pos_center =
                    (relative_chunk_pos.as_vec3() + Vec3::splat(0.5)) * Chunk::SIDE as f32;

                if camera.is_sphere_in_frustum(relative_chunk_pos_center, CHUNK_RADIUS) {
                    callback(camera_chunk_pos + relative_chunk_pos);
                }
            }
        }
    }
}
