use web_sys::wasm_bindgen::JsValue;

use crate::{Message, Verbosity};

/// Logs a message using the borwser's console.
pub fn log(
    Message {
        file,
        line,
        verbosity,
        message,
        ..
    }: Message,
) {
    type LogFn = fn(&JsValue, &JsValue, &JsValue, &JsValue);

    let (f, verbosity, verbosity_color) = match verbosity {
        Verbosity::Error => (web_sys::console::error_4 as LogFn, "ERROR", "#f00"),
        Verbosity::Warning => (web_sys::console::warn_4 as LogFn, "WARNING", "#ff0"),
        Verbosity::Info => (web_sys::console::info_4 as LogFn, "INFO", "#00f"),
        Verbosity::Trace => (web_sys::console::log_4 as LogFn, "TRACE", "#222"),
    };

    let message = format!("%c{verbosity}%c  {message}  %c({file}:{line})");
    let css0 = format!("color: {verbosity_color}; font-weight: bold;");
    let css1 = "color: inherit; font-weight: inherit;";
    let css2 = "color: #888;";

    f(&message.into(), &css0.into(), &css1.into(), &css2.into());
}
