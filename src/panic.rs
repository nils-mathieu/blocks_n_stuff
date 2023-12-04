use std::panic::PanicInfo;

/// Displays a message box to the user with the provided message.
///
/// If the current platform does not support message boxes, this function does nothing.
///
/// This function is expected to block until the user closes the message box.
#[allow(unused_variables)]
#[cfg(not(debug_assertions))]
fn display_message_box(message: &str) {
    #[cfg(target_os = "windows")]
    {
        // TODO:
        //  It might be cool to use the HWND of the window we opened to display the message box.
        //  It's kinda tricky because the panic handler is not defined at all at the same place.
        //  Maybe global storage? But that would require to clean it up properly when the window
        //  is later destroyed. One alternative would be to install yet another custom panic
        //  handler *after* the window is created, but that would prevent us to display errors
        //  properly if the window creation fails.

        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};

        let text = message.encode_utf16().chain(Some(0)).collect::<Vec<_>>();

        // The error is ignored. If we can't event display a message box, there's nothing more we
        // can do.
        let _ = unsafe {
            MessageBoxW(
                0,
                text.as_ptr(),
                windows_sys::w!("Unexpected panic :("),
                MB_ICONERROR,
            )
        };
    }
}

/// The custom panic hook.
///
/// This panic hook is responsible for printing the panic message to the console, as well as opening
/// a window on platform that support it.
fn custom_panic_hook(info: &PanicInfo) {
    // Get the panic message out of the payload.
    let mut message = info
        .payload()
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
        .unwrap_or("no further information")
        .to_owned();

    if let Some(location) = info.location() {
        use std::fmt::Write;

        let _ = write!(
            message,
            " (at {}:{}:{})",
            location.file(),
            location.line(),
            location.column()
        );
    }

    // Display the message to the user using the console.
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), "\x1B[1;31mpanic\x1B[0m: {message}");
    }

    // Except on WASM, where we use the browser's console.
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::js_sys::wasm_bindgen::JsValue;

        let message = format!("%cPANIC%c  {}", message);
        let css1 = "color: white; font-weight: bold; background-color: red;";
        let css2 = "color: inherit; font-weight: normal; background-color: inherit;";
        web_sys::console::error_3(
            &JsValue::from(message),
            &JsValue::from(css1),
            &JsValue::from(css2),
        );
    }

    // Display the message to the user using a message box.
    // Only do that in release though, because it's a bit annoying when debugging.
    #[cfg(not(debug_assertions))]
    {
        // This `Once` is used to avoid showing multiple message boxes if multiple threads panic
        // at the same time.
        // The `call_once` method will block if another thread is already calling it, and will
        // only unblock (but not call the closure) when the other thread is done (the user
        // closed the message box).
        static WINDOW_SHOWED_UP: std::sync::Once = std::sync::Once::new();
        WINDOW_SHOWED_UP.call_once(|| display_message_box(&message));
    }
}

/// Installs the custom panic hook.
pub fn install_custom_panic_hook() {
    bns_log::trace!("installing custom panic hook...");
    std::panic::set_hook(Box::new(custom_panic_hook));
}
