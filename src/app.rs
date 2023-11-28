use std::sync::Arc;

use glam::{Mat4, Quat, Vec2, Vec3};

use winit::event::KeyEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::KeyCode;
use winit::window::{Fullscreen, Window};

use crate::gfx::render_data::{FrameUniforms, RenderData, UniformBuffer};
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

    /// The uniform buffer that stores the frame-specific data.
    frame_uniforms: UniformBuffer<FrameUniforms>,

    /// The current state of the camera.
    camera: Camera,
}

impl App {
    /// Creates a new [`App`] instance.
    pub fn new(window: Arc<Window>) -> Self {
        let gpu = Gpu::new();
        let surface = Surface::new(gpu.clone(), window.clone());
        let renderer = Renderer::new(gpu.clone(), surface.format());
        let frame_uniforms = renderer.create_frame_uniform_buffer();

        Self {
            window,
            surface,
            renderer,
            frame_uniforms,
            camera: Camera::default(),
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

    /// Renders a frame to the window.
    pub fn render(&self) {
        // Write the frame-specific data to the uniform buffer.
        self.frame_uniforms.write(&FrameUniforms {
            camera: self.camera.matrix(),
        });

        let render_data = RenderData {
            frame_uniforms: &self.frame_uniforms,
        };

        self.renderer.render_to_surface(&self.surface, &render_data);
    }

    /// Advances the state of the application by one tick.
    pub fn tick(&mut self, _target: &Ctx, dt: f32) {
        self.camera.tick(dt);
    }
}

/// Stores the state of the camera.
#[derive(Default)]
pub struct Camera {
    /// Whether the user is currently pressing the "forward" key.
    pressing_forward: bool,
    /// Whether the user is currently pressing the "backward" key.
    pressing_backward: bool,
    /// Whether the user is currently pressing the "left" key.
    pressing_left: bool,
    /// Whether the user is currently pressing the "right" key.
    pressing_right: bool,
    /// Whether the user is currently pressing the "fly up" key.
    pressing_fly_up: bool,
    /// Whether the user is currently pressing the "fly down" key.
    pressing_fly_down: bool,
    /// The current movement input of the camera.
    ///
    /// The X component represents the horizontal movement, and the Y component represents the
    /// forward movement.
    movement_input: Vec2,
    /// The vertical movement input.
    vertical_movement_input: f32,
    /// The position of the camera in world-space coordinates.
    position: Vec3,
    /// The yaw of the camera, in radians.
    yaw: f32,
    /// The pitch of the camera, in radians.
    pitch: f32,
    /// The aspect ratio of the output display.
    aspect_ratio: f32,
}

impl Camera {
    /// The speed at which the camera moves, in units per second.
    pub const SPEED: f32 = 0.1;
    /// The speed at which the camera flies up/down.
    pub const FLY_SPEED: f32 = 0.1;
    /// The vertical field of view of the camera, in degrees.
    pub const FOV_Y: f32 = 90.0;

    /// Notifies the camera that the size of the output display has changed.
    pub fn notify_resized(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// Notifies the camera that a keyboard event has been received.
    pub fn notify_keyboard(&mut self, event: &KeyEvent) {
        if event.physical_key == KeyCode::KeyW {
            self.pressing_forward = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyS {
            self.pressing_backward = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyA {
            self.pressing_left = event.state.is_pressed();
        } else if event.physical_key == KeyCode::KeyD {
            self.pressing_right = event.state.is_pressed();
        } else if event.physical_key == KeyCode::Space {
            self.pressing_fly_up = event.state.is_pressed();
        } else if event.physical_key == KeyCode::ShiftLeft {
            self.pressing_fly_down = event.state.is_pressed();
        }

        self.movement_input = Vec2::ZERO;
        if self.pressing_forward {
            self.movement_input.y += 1.0;
        }
        if self.pressing_backward {
            self.movement_input.y -= 1.0;
        }
        if self.pressing_left {
            self.movement_input.x -= 1.0;
        }
        if self.pressing_right {
            self.movement_input.x += 1.0;
        }
        self.movement_input = self.movement_input.normalize_or_zero();

        self.vertical_movement_input = 0.0;
        if self.pressing_fly_up {
            self.vertical_movement_input += 1.0;
        }
        if self.pressing_fly_down {
            self.vertical_movement_input -= 1.0;
        }
    }

    /// Updates the state of the camera.
    pub fn tick(&mut self, dt: f32) {
        self.position += Quat::from_rotation_y(self.yaw)
            * Vec3::new(self.movement_input.x, 0.0, self.movement_input.y)
            * Self::SPEED
            * dt;

        self.position.y += self.vertical_movement_input * Self::FLY_SPEED * dt;
    }

    /// Computes the rotation of the camera.
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_x(self.pitch) * Quat::from_rotation_y(self.yaw)
    }

    /// Computes the forward vector of the camera.
    pub fn forward(&self) -> Vec3 {
        self.rotation() * Vec3::Z
    }

    /// Computes the matrix that transforms world-space coordinates into clip-space coordinates.
    pub fn matrix(&self) -> Mat4 {
        let perspective = Mat4::perspective_lh(Self::FOV_Y, self.aspect_ratio, 0.1, 100.0);
        let view = Mat4::look_to_lh(self.position, self.forward(), Vec3::Y);
        perspective * view
    }
}
