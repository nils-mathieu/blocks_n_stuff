#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod game;
mod panic;
mod world;

/// The glorious entry point of the program!
///
/// No shit, Sherlock!
fn main() {
    panic::install_custom_panic_hook();
    app::run();
}

#[cfg(target_arch = "wasm32")]
mod wasm {

    use web_sys::wasm_bindgen;
    use web_sys::wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    #[allow(clippy::main_recursion)]
    fn wasm_entry_point() {
        super::main();
    }
}
