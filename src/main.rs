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
