//! This module contains the game-related logic, aggregating all the other modules.

use std::sync::Arc;

use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

use crate::gfx::render_data::{
    FrameUniforms, QuadInstance, RenderData, UniformBuffer, VertexBuffer,
};
use crate::gfx::Renderer;
use crate::window::UserEvent;

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

    /// The uniform buffer that stores the frame-specific data.
    frame_uniforms: UniformBuffer<FrameUniforms>,
    /// The quads to draw.
    quads: VertexBuffer<QuadInstance>,

    /// The current state of the camera.
    camera: Camera,
}

impl App {
    /// Creates a new [`App`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(window.clone());
        let frame_uniforms = renderer.create_frame_uniform_buffer();

        let mut quads_buf = Vec::new();
        for x in 0..31 {
            for z in 0..31 {
                let pos =
                    QuadInstance::from_x(x) | QuadInstance::from_y(0) | QuadInstance::from_z(z);

                quads_buf.extend_from_slice(&[
                    pos | QuadInstance::X,
                    pos | QuadInstance::NEG_X,
                    pos | QuadInstance::Y,
                    pos | QuadInstance::NEG_Y,
                    pos | QuadInstance::Z,
                    pos | QuadInstance::NEG_Z,
                ]);
            }
        }

        let mut quads = VertexBuffer::new(renderer.gpu().clone(), quads_buf.len() as _);
        quads.write(&quads_buf);

        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
            .expect("failed to grab the mouse cursor");
        window.set_cursor_visible(false);

        Self {
            window,
            renderer,
            frame_uniforms,
            camera: Camera::default(),
            quads,
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

    /// Renders a frame to the window.
    pub fn render(&mut self) {
        // Write the frame-specific data to the uniform buffer.
        self.frame_uniforms.write(&FrameUniforms {
            camera: self.camera.matrix(),
        });

        let render_data = RenderData {
            frame_uniforms: &self.frame_uniforms,
            quads: &self.quads,
        };

        self.renderer.render(&render_data);
    }

    /// Advances the state of the application by one tick.
    pub fn tick(&mut self, _target: &Ctx, dt: f32) {
        self.camera.tick(dt);
    }
}
