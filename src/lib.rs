//! This crate contains a command line tool and its library
//! to change input sources of display monitors with [DDC/CI].
//!
//! * The [`Cli`] struct provides the command line tool.
//! * The [`Monitor`] struct provides functions to
//!   change input sources of display monitors.
//!
//! [DDC/CI]: https://en.wikipedia.org/wiki/Display_Data_Channel
mod cli;
pub use cli::*;

mod input_source;
pub use input_source::*;

mod monitor;
pub use monitor::*;
