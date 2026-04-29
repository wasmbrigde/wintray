#[cfg(not(windows))]
compile_error!("Этот проект поддерживает только Windows.");

pub mod config;
pub mod engine;
pub mod tray;

pub use engine::{WintrayApp, WintrayAppBuilder};
