//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;

use bns_core::{Chunk, TextureId};
use bns_render::data::{ChunkUniforms, FrameUniforms, RenderDataStorage};
use bns_render::{Renderer, RendererConfig, Surface, TextureAtlasConfig, TextureFormat};
use glam::{IVec3, Vec3};
use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

use crate::window::UserEvent;
use crate::world::{ChunkPos, World};
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
    /// The surface on which things are rendered.
    surface: Surface<'static>,
    /// The renderer contains the resources required to render things using GPU resources.
    renderer: Renderer,
    /// Some storage to efficiently create [`RenderData`](bns_render::data::RenderData) instances.
    render_data_storage: RenderDataStorage,

    /// The current state of the camera.
    camera: Camera,

    /// The world that contains all the chunks.
    world: World,
}

impl App {
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
            render_data_storage: RenderDataStorage::new(&renderer),
            window,
            surface,
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
    pub fn notify_resized(&mut self, width: u32, height: u32) {
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

        self.camera.notify_keyboard(event);
    }

    /// Notifies the application that the mouse has moved.
    ///
    /// Note that the provided coordinates are not in pixels, but instead are an arbitrary
    /// value relative to the last reported mouse position.
    pub fn notify_mouse_moved(&mut self, _taget: &Ctx, dx: f64, dy: f64) {
        self.camera.notify_mouse_moved(dx, dy);
    }

    /// Renders a frame to the window.
    pub fn render(&mut self) {
        // TODO: print quad count in debug info.

        let Some(frame) = self.surface.acquire_image() else {
            return;
        };
        let mut render_data = self.render_data_storage.build();
        render_data.clear_color([0.0, 0.0, 1.0, 1.0]);
        let view = self.camera.view_matrix();
        let projection = self.camera.projection_matrix();
        render_data.frame_uniforms(FrameUniforms {
            inverse_projection: projection.inverse(),
            projection,
            inverse_view: view.inverse(),
            view,
        });
        chunks_in_frustum(&self.camera, |chunk_pos| {
            if let Some(chunk) = self.world.get_existing_chunk(chunk_pos) {
                if let Some(buffer) = &chunk.geometry.quads {
                    render_data.add_quad_vertices(
                        ChunkUniforms {
                            position: chunk_pos,
                        },
                        buffer,
                    );
                }
            }
        });
        self.renderer.render(frame.target(), &render_data);
        frame.present();
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
    const HORIZONTAL_RENDER_DISTANCE: i32 = 6;
    const VERTICAL_RENDER_DISTANCE: i32 = 6;
    const CHUNK_RADIUS: f32 = (Chunk::SIDE as f32) * 0.8660254; // sqrt(3) / 2

    fn coord_to_chunk(coord: f32) -> i32 {
        if coord >= 0.0 {
            coord as i32 / Chunk::SIDE
        } else {
            coord as i32 / Chunk::SIDE - 1
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
                let relative_chunk_pos_center = (relative_chunk_pos.as_vec3() + Vec3::splat(0.5))
                    * Chunk::SIDE as f32
                    - (camera.position() - camera_chunk_pos.as_vec3() * Chunk::SIDE as f32);

                if camera.is_sphere_in_frustum(relative_chunk_pos_center, CHUNK_RADIUS) {
                    callback(camera_chunk_pos + relative_chunk_pos);
                }
            }
        }
    }
}

fn load_texture_atlas() -> TextureAtlasConfig<'static> {
    let mut data = Vec::new();
    let mut count = 0;
    let mut metadata = None;

    for texture_id in TextureId::all() {
        let path = format!("assets/{}.png", texture_id.file_name());
        println!("loading texture: {}", path);
        let mut image = bns_image::Image::load_png(std::fs::File::open(path).unwrap()).unwrap();
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
            bns_image::ColorSpace::Linear => TextureFormat::Rgba8Unorm,
        },
    }
}
