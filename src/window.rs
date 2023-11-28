use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowBuilder};

/// The type of the user events dispatched through the event loop.
///
/// Right now, no custom events are used.
enum UserEvent {}

/// This function is responsible for creating a window, as well as dispatching the events it
/// receives from the underlying platform to the application state.
///
/// # Panics
///
/// This function panics if it fails to initialize the window or the event loop. Additionally, if
/// an error occurs while running the event loop, the function panics even if an exit code is
/// requested.
pub fn run() {
    let event_loop = create_event_loop();
    let window = create_window(&event_loop);

    // TODO: tick the application once to render a frame, and only then make the window visible.
    window.set_visible(true);

    event_loop
        .run(move |event, target| match event {
            Event::AboutToWait => {
                // This is where the main application logic should run.
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    // TODO: remove this when a menu is implemented to exit the application.
                    // The key to open the menu will probably be Escape key anyway so I won't
                    // miss this.
                    if event.state.is_pressed() && event.physical_key == KeyCode::Escape {
                        target.exit();
                    }
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
fn create_window(event_loop: &EventLoop<UserEvent>) -> Window {
    WindowBuilder::new()
        .with_title("Blocks 'n Stuff")
        .with_min_inner_size(PhysicalSize::new(300, 300))
        .with_visible(false)
        .build(event_loop)
        .expect("failed to create the `winit` window")
}
