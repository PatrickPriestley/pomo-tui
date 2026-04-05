#![allow(unknown_lints)]
#![allow(clippy::manual_is_multiple_of)]

#[cfg(feature = "audio")]
pub mod audio;
pub mod config;
pub mod core;
pub mod integrations;
pub mod persistence;
pub mod tui;
pub mod utils;
