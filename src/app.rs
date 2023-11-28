use std::sync::Arc;

use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{Fullscreen, Window};

use crate::gfx::{Gpu, Renderer, Surface};
use crate::window::UserEvent;

/// The context passed to the functions of [`App`] to control the event loop.
type Ctx = EventLoopWindowTarget<UserEvent>;

/// Contains the state of the application.
pub struct App {
    /// A handle to the window that has been opened for the application.
    ///
    /// This can be used to control it.
    window: Arc<Window>,
    /// The surface is responsible for presenting rendered frames to the window.
    surface: Surface,
    /// The renderer contains the resources required to render things using GPU resources.
    renderer: Renderer,
    /// An open connection with the Graphics Processing Unit that has been selected for use.
    gpu: Arc<Gpu>,
}

impl App {
    /// Creates a new [`App`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let gpu = Gpu::new();
        let surface = Surface::new(gpu.clone(), window.clone());
        let renderer = Renderer::new(gpu.clone(), surface.format());

        Self {
            window,
            surface,
            renderer,
            gpu,
        }
    }

    /// Notifies the application that the window has been requested to close.
    pub fn notify_close_requested(&mut self, target: &Ctx) {
        target.exit();
    }

    /// Notifies the application that the size of the window on which it is drawing stuff has
    /// changed.
    pub fn notify_resized(&mut self, _target: &Ctx, width: u32, height: u32) {
        self.surface.notify_resized(width, height);
    }

    /// Notifies the application that a keyboard event has been received.
    pub fn notify_keyboard(&mut self, target: &Ctx, event: KeyEvent) {
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
    }

    /// Renders a frame to the window.
    pub fn render(&self) {
        self.renderer.render_to_surface(&self.surface);
    }

    /// Advances the state of the application by one tick.
    pub fn tick(&mut self, _target: &Ctx) {}
}
