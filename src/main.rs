mod panic;
mod window;

/// The glorious entry point of the program!
///
/// No shit, Sherlock!
fn main() {
    panic::install_custom_panic_hook();
    window::run();
}
