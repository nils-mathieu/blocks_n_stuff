#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod panic;
mod window;
mod world;

/// The glorious entry point of the program!
///
/// No shit, Sherlock!
fn main() {
    panic::install_custom_panic_hook();
    window::run();
}

#[cfg(target_arch = "wasm32")]
mod wasm {

    use web_sys::wasm_bindgen;
    use web_sys::wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    #[allow(clippy::main_recursion)]
    fn wasm_entrp_point() {
        super::main();
    }
}
