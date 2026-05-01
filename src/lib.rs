//! `wintray` is a lightweight framework for building Windows tray applications with an embedded web UI.
//! It integrates a system tray icon, a custom context menu, and an Axum-based web server
//! with built-in TLS support.

#[cfg(not(windows))]
compile_error!("wintray currently only supports Windows.");

pub mod config;
pub mod engine;
pub mod tray;

pub use engine::{WintrayApp, WintrayAppBuilder};
