//! A simple logging library for the needs of Blocks 'n Stuff.

use std::fmt::Arguments;

#[cfg_attr(target_arch = "wasm32", path = "wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "std.rs")]
mod imp;

/// A verbosity level for a [`Message`].
///
/// # Remarks
///
/// The ordering of the verbosity levels is in *increasing verbosity*, meaning that
/// [`Error`] is the *least verbose*, and [`Trace`] is the *most verbose*.
///
/// This is useful for filtering messages based on their verbosity level.
///
/// [`Error`]: Verbosity::Error
/// [`Trace`]: Verbosity::Trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verbosity {
    /// The minimum verbosity level possible.
    ///
    /// This is used to indicate that a message notifies the user of a fatal error. This is
    /// not necessarily a panic, but it's a situtation that prevents at least part of the program
    /// from working correctly.
    Error,
    /// A verbosity level that indicates that a message notifies the user of a warning.
    ///
    /// Warnings are usually errors from which the program has recovered by itself, but which
    /// may indicate that something is wrong.
    Warning,
    /// A verbosity level that indicates that a message notifies the user of an information.
    ///
    /// Those information are useful most of the time, but they do not indicate that something
    /// is wrong.
    Info,
    /// A verbosity level that indicates that a message notifies the user of a debug information.
    ///
    /// Those information are useful for debugging purposes, but they are not useful for the
    /// end-user.
    Trace,
}

/// A message that can be logged.
pub struct Message<'a> {
    /// The name of the file in which the message was logged.
    pub file: &'static str,
    /// The line at which the message was logged.
    pub line: u32,
    /// The column at which the message was logged.
    pub column: u32,
    /// The verbosity level of the message.
    pub verbosity: Verbosity,
    /// The module in which the message was logged.
    pub module: &'static str,
    /// The message itself.
    pub message: Arguments<'a>,
}

impl<'a> Message<'a> {
    /// Logs this message.
    pub fn log(self) {
        imp::log(self);
    }
}

/// Creates a [`Message`] instance with the current invoking location.
#[macro_export]
macro_rules! message {
    ($verbosity:expr, $($args:tt)*) => {
        $crate::Message {
            file: ::core::file!(),
            line: ::core::line!(),
            column: ::core::column!(),
            verbosity: $verbosity,
            module: ::core::module_path!(),
            message: ::core::format_args!($($args)*),
        }
    };
}

/// Logs a message with the current invoking location.
///
/// # Remarks
///
/// This macro is basically equivalent to calling [`Message::log`] on the result of
/// [`message!`].
#[macro_export]
macro_rules! log {
    ($verbosity:expr, $($args:tt)*) => {
        $crate::Message::log($crate::message!($verbosity, $($args)*))
    };
}

/// Logs a message with the current invoking location, with a verbosity level of
/// [`Verbosity::Error`].
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Error, $($args)*)
    };
}

/// Logs a message with the current invoking location, with a verbosity level of
/// [`Verbosity::Warning`].
#[macro_export]
macro_rules! warning {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Warning, $($args)*)
    };
}

/// Logs a message with the current invoking location, with a verbosity level of
/// [`Verbosity::Info`].
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Info, $($args)*)
    };
}

/// Logs a message with the current invoking location, with a verbosity level of
/// [`Verbosity::Trace`].
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::log!($crate::Verbosity::Trace, $($args)*)
    };
}
