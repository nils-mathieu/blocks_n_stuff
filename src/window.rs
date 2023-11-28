use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Window, WindowBuilder};

use crate::app::App;

/// The type of the user events dispatched through the event loop.
///
/// Right now, no custom events are used.
pub enum UserEvent {}

/// This function is responsible for creating a window, as well as dispatching the events it
/// receives from the underlying platform to the application state.
///
/// # Panics
///
/// This function panics if it fails to initialize the window or the event loop. Additionally, if
/// an error occurs while running the event loop, the function panics even if an exit code is
/// requested.
#[allow(clippy::collapsible_match, clippy::single_match)]
pub fn run() {
    let event_loop = create_event_loop();
    let window = create_window(&event_loop);
    let mut app = App::new(window.clone());

    app.render();
    window.set_visible(true);

    event_loop
        .run(move |event, target| match event {
            Event::AboutToWait => {
                // TODO: properly compute the delta time.
                // Note that this must take in account that sometimes the `AboutToWait` event is
                // never fired. Maybe simply giving an upper bound to the delta time is enough?
                // The rule is pretty simple: the user must not see things accelerate. Only slow
                // down in the worst case.
                let dt = 1.0;

                // This is where the main application logic should run.
                app.tick(target, dt);
                app.render();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    app.notify_close_requested(target);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    app.notify_keyboard(target, &event);
                }
                WindowEvent::Resized(s) => {
                    app.notify_resized(target, s.width, s.height);
                }
                WindowEvent::RedrawRequested => {
                    app.render();
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                    app.notify_mouse_moved(target, dx, dy);
                }
                _ => (),
            },
            _ => (),
        })
        .expect("failed to run the `winit` event loop");
}

/// Creates an event loop with the appropriate settings.
///
/// # Panics
///
/// This function panics if the event loop cannot be created.
fn create_event_loop() -> EventLoop<UserEvent> {
    let event_loop = EventLoopBuilder::with_user_event()
        .build()
        .expect("failed to create the `winit` event loop");

    // Prevent the winit event loop from blocking the thread when no events are available for any
    // of the windows it manages. We want to run the main application loop alongside the event loop,
    // and this requires that the event loop does not block the thread.
    //
    // Instead of having the thread block for events, it will block for the display's vertical
    // blanking period (if VSync is enabled).
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
}

/// Creates a window with the appropriate settings for the application.
///
/// Most notably the window is initially created invisible to ensure that no garbage or flickering
/// is visible from the user's standpoint.
///
/// # Panics
///
/// This function panics if the window cannot be created.
fn create_window(event_loop: &EventLoop<UserEvent>) -> Arc<Window> {
    WindowBuilder::new()
        .with_title("Blocks 'n Stuff")
        .with_min_inner_size(PhysicalSize::new(300, 300))
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_visible(false)
        .build(event_loop)
        .expect("failed to create the `winit` window")
        .into()
}
