use crate::{Message, Verbosity};
use std::io::Write;

/// Logs a message to the standard error stream.
pub fn log(
    Message {
        file,
        line,
        verbosity,
        message,
        ..
    }: Message,
) {
    let prefix = match verbosity {
        Verbosity::Error => "\x1B[1;31mERROR\x1B[0m  ",
        Verbosity::Warning => "\x1B[1;33mWARNING\x1B[0m",
        Verbosity::Info => "\x1B[1;34mINFO\x1B[0m   ",
        Verbosity::Trace => "\x1B[1;30mTRACE\x1B[0m  ",
    };

    let _ = writeln!(
        std::io::stderr().lock(),
        "{prefix}{message} \x1B[2;90m(at {file}:{line})\x1B[0m"
    );
}
