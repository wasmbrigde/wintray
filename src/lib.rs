//! `wintray` is a lightweight framework for building Windows tray applications with an embedded web UI.
//! It integrates a system tray icon, a custom context menu, and an Axum-based web server
//! with built-in TLS support.

#[cfg(not(windows))]
compile_error!("wintray currently only supports Windows.");

pub mod assets;
pub mod config;
pub mod engine;
pub mod tray;

// Re-export common dependencies to reduce boilerplate in projects
pub use axum;
pub use askama;
pub use rust_embed;
pub use serde;
pub use mime_guess;
pub use rustls;



/// A collection of commonly used types and traits for building Wintray applications.
pub mod exports {
    pub use axum::{
        self,
        Router,
        body::Bytes,
        routing::{get, post, any},
        extract::{State, Form, Path, Query, Multipart},
        response::{Html, IntoResponse, Response},
        http::{StatusCode, HeaderMap, Method},
    };
    pub use askama::Template;
    pub use rust_embed::RustEmbed;
    pub use serde::{Serialize, Deserialize};
    pub use crate::wintray_template;
    pub use crate::wintray_assets;
}

pub use engine::{WintrayApp, WintrayAppBuilder};
pub use wintray_macros::{wintray_template, wintray_assets};
