/// The configuration of an [`App`](super::App).
pub struct Config<'a> {
    /// The title of the window that will be created for the application.
    ///
    /// This is the text that will be displayed in the title bar of the window.
    pub title: &'a str,
    /// The initial size of the window.
    ///
    /// If left unset, a platform-specific default value is used.
    pub size: Option<(u32, u32)>,
    /// The minimum size of the window.
    pub min_size: (u32, u32),
    /// Whether the window should start in fullscreen mode.
    pub fullscreen: bool,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            title: "My Awesome Application",
            min_size: (400, 400),
            fullscreen: false,
            size: None,
        }
    }
}
