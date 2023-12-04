//! A thin abstraction over [`winit`].

use std::sync::Arc;

use raw_window_handle as rwh;

mod config;
mod ctx;

pub use config::*;
pub use ctx::*;

mod event_loop;

/// an opaque window object that guarantees that the window is valid while the object is alive.
pub struct OpaqueWindow(Arc<winit::window::Window>);

impl rwh::HasWindowHandle for OpaqueWindow {
    #[inline]
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        self.0.window_handle()
    }
}

impl rwh::HasDisplayHandle for OpaqueWindow {
    #[inline]
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        self.0.display_handle()
    }
}

/// Represents an application that's ready to run.
pub struct App {
    event_loop: winit::event_loop::EventLoop<event_loop::UserEvent>,
    window: Arc<winit::window::Window>,
}

impl App {
    /// Creates a new [`App`] instance with the given [`Config`].
    pub fn new(config: Config) -> Self {
        let event_loop = event_loop::create_event_loop();
        let window = event_loop::create_window(&event_loop, config);
        Self { event_loop, window }
    }

    /// Returns an [`OpaqueWindow`] that guarantees that the window is valid while the object is
    /// alive.
    #[inline]
    pub fn opaque_window(&self) -> OpaqueWindow {
        OpaqueWindow(self.window.clone())
    }

    /// Runs the application.
    pub fn run<F>(self, tick: F)
    where
        F: FnMut(&mut Ctx),
    {
        event_loop::run(self.event_loop, self.window, tick)
    }
}

impl rwh::HasWindowHandle for App {
    #[inline]
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        self.window.window_handle()
    }
}

impl rwh::HasDisplayHandle for App {
    #[inline]
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        self.window.display_handle()
    }
}
