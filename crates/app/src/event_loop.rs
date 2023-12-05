use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::ElementState::{Pressed, Released};
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Fullscreen, Window, WindowBuilder};

use crate::{Config, Ctx};

/// The user event type used by the event loop.
pub enum UserEvent {}

/// Creates an event loop with the appropriate settings.
///
/// # Panics
///
/// This function panics if the event loop cannot be created.
pub fn create_event_loop() -> EventLoop<UserEvent> {
    bns_log::trace!("creating the winit event loop...");

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
pub fn create_window(event_loop: &EventLoop<UserEvent>, config: Config) -> Arc<Window> {
    bns_log::trace!("creating the winit window...");

    let mut builder = WindowBuilder::new()
        .with_title(config.title)
        .with_visible(false)
        .with_min_inner_size(PhysicalSize::<u32>::from(config.min_size))
        .with_fullscreen(config.fullscreen.then_some(Fullscreen::Borderless(None)));

    if let Some(size) = config.size {
        builder = builder.with_inner_size(PhysicalSize::<u32>::from(size));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowBuilderExtWebSys;

        // Request winit to add a canvas in the DOM for us.
        builder = builder.with_append(true);
    }

    builder
        .build(event_loop)
        .expect("failed to create the `winit` window")
        .into()
}

/// Runs the event loop until completion.
///
/// # Panics
///
/// This function panics if the event loop cannot be run.
#[allow(clippy::collapsible_match, clippy::single_match)]
pub fn run<F>(event_loop: EventLoop<UserEvent>, window: Arc<Window>, mut tick: F)
where
    F: FnMut(&mut Ctx),
{
    let mut ctx = Ctx::new(window);

    // Tick the application once, hoping that it will try to draw something to the window
    // before we actually show it.
    tick(&mut ctx);

    bns_log::trace!("running the winit event loop...");
    ctx.winit_window().set_visible(true);
    event_loop
        .run(move |event, target| match event {
            Event::AboutToWait => {
                // Because we made sure to set the control flow to `Poll`, we know that the
                // `AboutToWait` event won't actually indicate that the thread is about to block.
                // Instead, it's used as a way to detect when no more events are pending for any
                // of the windows managed by the event loop (we only have one).

                // We want to block the thread until the next vertical blanking period, so that
                // we don't waste CPU cycles rendering frames that will never be displayed (this
                // is done automatically by the GPU on most platform, but not web!)
                ctx.winit_window().request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => ctx.notify_close_requested(),
                WindowEvent::Resized(s) => ctx.notify_resized(s.width, s.height),
                WindowEvent::MouseInput { state, button, .. } => match state {
                    Pressed => ctx.notify_button_pressed(button.into()),
                    Released => ctx.notify_button_released(button.into()),
                },
                WindowEvent::KeyboardInput { event, .. } => {
                    match event.state {
                        Pressed => {
                            if !event.repeat {
                                ctx.notify_button_pressed(event.logical_key.into());
                                ctx.notify_button_pressed(event.physical_key.into());
                            }
                        }
                        Released => {
                            ctx.notify_button_released(event.logical_key.into());
                            ctx.notify_button_released(event.physical_key.into());
                        }
                    }

                    if let Some(txt) = event.text {
                        ctx.notify_typed(&txt)
                    }
                }
                WindowEvent::Focused(yes) => ctx.notify_focus_changed(yes),
                WindowEvent::RedrawRequested => {
                    ctx.notify_start_of_tick();

                    tick(&mut ctx);

                    if ctx.closing() {
                        bns_log::trace!("closing the winit event loop...");
                        target.exit();
                        return;
                    }

                    ctx.notify_end_of_tick();
                }
                WindowEvent::MouseWheel { delta, .. } => ctx.notify_mouse_scrolled(delta),
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => ctx.notify_mouse_moved(dx, dy),
                _ => (),
            },
            _ => (),
        })
        .expect("failed to run the winit event loop");
}
