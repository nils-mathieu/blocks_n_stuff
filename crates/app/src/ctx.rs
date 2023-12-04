use std::hash::BuildHasherDefault;
use std::sync::Arc;
use std::time::Duration;

use hashbrown::HashMap;
use rustc_hash::FxHasher;
use winit::window::{CursorGrabMode, Fullscreen, Window};

pub use winit::keyboard::{KeyCode, NamedKey, NativeKey, NativeKeyCode, SmolStr};

/// Contains the application context, which can be used to access the window and other
/// related resources.
pub struct Ctx {
    /// The winit window.
    window: Arc<Window>,

    /// Whether the event loop has been requested to close.
    closing: bool,

    /// Whether the window has been resized since the last tick.
    just_resized: bool,
    /// The current size of the window's client area.
    size: (u32, u32),

    /// The current state of the buttons.
    buttons: HashMap<AnyButton, ButtonState, BuildHasherDefault<FxHasher>>,

    /// The amount of movement accumulated by the mouse since the last tick.
    mouse_delta: (f64, f64),

    /// Whether the window currently has focus.
    focused: bool,
    /// Whether the window has just gained or lost focus.
    focus_just_changed: bool,

    /// Whether the window is currently in fullscreen mode.
    is_fullscreen: bool,

    /// The text that was typed since the last tick.
    typed: String,

    /// The clock that's used to compute times.
    clock: quanta::Clock,
    /// The time at which the application started.
    initial_instant: quanta::Instant,
    /// The time at which the last tick started.
    last_tick_instant: quanta::Instant,
    /// The amount of time elapsed since the application started.
    since_startup: Duration,
    /// The amount of time elapsed since the start of the last tick.
    since_last_tick: Duration,
    /// The delta time, converted as a `f32` to avoid converting it every time it's used.
    delta_seconds: f32,
}

impl Ctx {
    /// Creates a new [`Ctx`] instance with the given [`Window`].
    pub(crate) fn new(window: Arc<Window>) -> Self {
        let clock = quanta::Clock::new();
        let now = clock.now();

        Self {
            closing: false,
            just_resized: true,
            size: window.inner_size().into(),
            buttons: HashMap::default(),
            mouse_delta: (0.0, 0.0),
            focused: true,
            focus_just_changed: false,
            is_fullscreen: window.fullscreen().is_some(),
            window,
            typed: String::new(),
            initial_instant: now,
            last_tick_instant: now,
            since_startup: Duration::ZERO,
            since_last_tick: Duration::ZERO,
            delta_seconds: 0.0,
            clock,
        }
    }

    /// Returns the raw winit window.
    #[inline]
    pub(crate) fn winit_window(&self) -> &Window {
        &self.window
    }

    /// Notifies the context that the window it controls has been resized.
    #[inline]
    pub(crate) fn notify_resized(&mut self, w: u32, h: u32) {
        self.just_resized = true;
        self.size = (w, h);
    }

    /// Notifies the context that the event loop has been requested to close by the user.
    #[inline]
    pub(crate) fn notify_close_requested(&mut self) {
        self.closing = true;
    }

    /// Notifies the context that the mouse has moved.
    #[inline]
    pub(crate) fn notify_mouse_moved(&mut self, dx: f64, dy: f64) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    /// Returns the state of the requested button, if present.
    fn button(&self, btn: AnyButton) -> ButtonState {
        self.buttons.get(&btn).map_or(ButtonState::IDLE, |s| *s)
    }

    /// Returns the state of the requested button, or inserts a new one if not present.
    fn button_mut(&mut self, btn: AnyButton) -> &mut ButtonState {
        self.buttons.entry(btn).or_insert(ButtonState::IDLE)
    }

    /// Notifies the context that the requested button has been pressed.
    #[inline]
    pub(crate) fn notify_button_pressed(&mut self, btn: AnyButton) {
        self.button_mut(btn).notify_pressed();
    }

    /// Notifies the context that the requested button has been released.
    #[inline]
    pub(crate) fn notify_button_released(&mut self, btn: AnyButton) {
        self.button_mut(btn).notify_released();
    }

    /// Notifies the context that the window has gained or lost focus.
    #[inline]
    pub(crate) fn notify_focus_changed(&mut self, focused: bool) {
        self.focused = focused;
        self.focus_just_changed = true;
    }

    /// Notifies the context that the text has been typed.
    #[inline]
    pub(crate) fn notify_typed(&mut self, text: &str) {
        self.typed.push_str(text);
    }

    /// Notifies the context that the tick function has started.
    pub(crate) fn notify_start_of_tick(&mut self) {
        let now = self.clock.now();
        self.since_last_tick = now - self.last_tick_instant;
        self.since_startup = now - self.initial_instant;
        self.delta_seconds = self.since_last_tick.as_secs_f32();
        self.last_tick_instant = now;
    }

    /// Notifies the context that the tick function has returned. New events are about to be
    /// processed.
    pub(crate) fn notify_end_of_tick(&mut self) {
        self.just_resized = false;
        self.mouse_delta = (0.0, 0.0);
        self.focus_just_changed = false;
        self.typed.clear();
        self.buttons
            .values_mut()
            .for_each(ButtonState::notify_end_of_tick);
    }

    /// Returns whether the application has been requested to close itself.
    ///
    /// If this boolean remains `true` at the end of the current tick, the application will
    /// actually close itself.
    ///
    /// It is possible to cancel the event by calling [`cancel_closing`].
    ///
    /// [`cancel_closing`]: Self::cancel_closing
    #[inline]
    pub fn closing(&self) -> bool {
        self.closing
    }

    /// Requests the application to close.
    #[inline]
    pub fn close(&mut self) {
        self.closing = true;
    }

    /// Cancels a closing request initiated by [`close`].
    ///
    /// [`close`]: Self::close
    #[inline]
    pub fn cancel_closing(&mut self) {
        self.closing = false;
    }

    /// Returns the current size of the window.
    ///
    /// It is possible to query whether the size has changed since the last frame by checking
    /// th return value of [`just_resized`].
    ///
    /// [`resized`]: Self::just_resized
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns the current width of the window.
    ///
    /// It is possible to query whether the size has changed since the last frame by checking
    /// th return value of [`just_resized`].
    ///
    /// [`resized`]: Self::just_resized
    #[inline]
    pub fn width(&self) -> u32 {
        self.size.0
    }

    /// Returns the current height of the window.
    ///
    /// It is possible to query whether the size has changed since the last frame by checking
    /// the return value of [`just_resized`].
    ///
    /// [`resized`]: Self::just_resized
    #[inline]
    pub fn height(&self) -> u32 {
        self.size.1
    }

    /// Returns whether the window has been resized since the last tick.
    ///
    /// The new size of the window can be queried by calling [`size`].
    ///
    /// [`size`]: Self::size
    #[inline]
    pub fn just_resized(&self) -> bool {
        self.just_resized
    }

    /// Returns whether the requested button has been pressed since the last frame.
    #[inline]
    pub fn just_pressed(&self, btn: impl Into<AnyButton>) -> bool {
        self.button(btn.into()).just_pressed()
    }

    /// Returns whether the requested button has been released since the last frame.
    #[inline]
    pub fn just_released(&self, btn: impl Into<AnyButton>) -> bool {
        self.button(btn.into()).just_released()
    }

    /// Returns whether the requested button is currently pressed.
    #[inline]
    pub fn pressing(&self, btn: impl Into<AnyButton>) -> bool {
        self.button(btn.into()).pressed()
    }

    /// The relative position of the mouse since the last tick.
    #[inline]
    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    /// The relative position of the mouse on the X axis since the last tick.
    #[inline]
    pub fn mouse_delta_x(&self) -> f64 {
        self.mouse_delta.0
    }

    /// The relative position of the mouse on the Y axis since the last tick.
    #[inline]
    pub fn mouse_delta_y(&self) -> f64 {
        self.mouse_delta.1
    }

    /// Returns whether the window is currently is fullscreen mode.
    #[inline]
    pub fn fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    /// Sets whether the window should be in fullscreen mode.
    #[inline]
    pub fn set_fullscreen(&mut self, yes: bool) {
        self.window
            .set_fullscreen(yes.then_some(Fullscreen::Borderless(None)));
    }

    /// Returns the text that has been typed since the last tick.
    #[inline]
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Attempt to grab the cursor, hiding it and locking it to the window.
    pub fn grab_cursor(&mut self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
            .expect("failed to grab the cursor");
        self.window.set_cursor_visible(false);
    }

    /// Returns the amount of time elapsed since the application started.
    #[inline]
    pub fn since_startup(&self) -> Duration {
        self.since_startup
    }

    /// Returns the amount of time elapsed since the start of the last tick.
    #[inline]
    pub fn since_last_tick(&self) -> Duration {
        self.since_last_tick
    }

    /// Returns the amount of time elapsed since the start of the last tick, converted to seconds.
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

pub use winit::event::MouseButton;
pub use winit::keyboard::{Key, PhysicalKey};

/// Any kind of button that's supposed by the [`Ctx`] type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnyButton {
    /// A mouse button.
    Mouse(MouseButton),
    /// A logical keyboard key.
    LogicalKey(Key),
    /// A physical keyboard key.
    PhysicalKey(PhysicalKey),
}

impl From<MouseButton> for AnyButton {
    #[inline]
    fn from(btn: MouseButton) -> Self {
        Self::Mouse(btn)
    }
}

impl From<Key> for AnyButton {
    #[inline]
    fn from(btn: Key) -> Self {
        Self::LogicalKey(btn)
    }
}

impl From<PhysicalKey> for AnyButton {
    #[inline]
    fn from(btn: PhysicalKey) -> Self {
        Self::PhysicalKey(btn)
    }
}

impl From<NamedKey> for AnyButton {
    #[inline]
    fn from(btn: NamedKey) -> Self {
        Key::from(btn).into()
    }
}

impl From<SmolStr> for AnyButton {
    fn from(value: SmolStr) -> Self {
        Key::Character(value).into()
    }
}

impl From<&str> for AnyButton {
    #[inline]
    fn from(value: &str) -> Self {
        SmolStr::from(value).into()
    }
}

impl From<char> for AnyButton {
    #[inline]
    fn from(value: char) -> Self {
        let mut buf = [0u8; 4];
        (*value.encode_utf8(&mut buf)).into()
    }
}

impl From<NativeKey> for AnyButton {
    #[inline]
    fn from(value: NativeKey) -> Self {
        Key::from(value).into()
    }
}

impl From<KeyCode> for AnyButton {
    #[inline]
    fn from(value: KeyCode) -> Self {
        PhysicalKey::from(value).into()
    }
}

impl From<NativeKeyCode> for AnyButton {
    #[inline]
    fn from(value: NativeKeyCode) -> Self {
        PhysicalKey::from(value).into()
    }
}

/// The flag type used to represent the state of a button.
#[derive(Debug, Clone, Copy)]
struct ButtonState(pub u8);

/// The flag used to indicate that a button has been pressed since the last frame.
const BUTTON_JUST_PRESSED: u8 = 1 << 0;
/// The flag used to indicate that a button has been released since the last frame.
const BUTTON_JUST_RELEASED: u8 = 1 << 1;
/// The flag used to indicate that a button is currently pressed.
const BUTTON_PRESSED: u8 = 1 << 2;

impl ButtonState {
    /// A button that's idle.
    pub const IDLE: Self = Self(0);

    /// Notifies this state that the button it represents has been pressed.
    #[inline]
    pub fn notify_pressed(&mut self) {
        self.0 |= BUTTON_JUST_PRESSED | BUTTON_PRESSED;
    }

    /// Notifies this state that the button it represents has been released.
    #[inline]
    pub fn notify_released(&mut self) {
        self.0 |= BUTTON_JUST_RELEASED;
        self.0 &= !BUTTON_PRESSED;
    }

    /// Returns whether the button it represents has been pressed since the last frame.
    #[inline]
    pub fn just_pressed(&self) -> bool {
        self.0 & BUTTON_JUST_PRESSED != 0
    }

    /// Returns whether the button it represents has been released since the last frame.
    #[inline]
    pub fn just_released(&self) -> bool {
        self.0 & BUTTON_JUST_RELEASED != 0
    }

    /// Returns whether the button it represents is currently pressed.
    #[inline]
    pub fn pressed(&self) -> bool {
        self.0 & (BUTTON_PRESSED | BUTTON_JUST_PRESSED) != 0
    }

    /// Notifies this state that the tick function has returned. New events are about to be
    /// processed.
    #[inline]
    pub fn notify_end_of_tick(&mut self) {
        self.0 &= !BUTTON_JUST_PRESSED;
        self.0 &= !BUTTON_JUST_RELEASED;
    }
}
