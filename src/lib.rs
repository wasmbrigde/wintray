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
pub use askama;
pub use mime_guess;
pub use poem;
pub use rust_embed;
pub use rustls;
pub use serde;

/// A collection of commonly used types and traits for building Wintray applications.
pub mod exports {
    pub use crate::wintray_assets;
    pub use crate::wintray_template;
    pub use askama::Template;
    pub use poem::{
        self, delete, get, handler, head,
        http::{HeaderMap, Method, StatusCode},
        patch, post, put,
        web::{Data, Form, Html, Multipart, Path, Query},
        Body, Endpoint, EndpointExt, IntoEndpoint, IntoResponse, Response, Route,
    };
    pub use rust_embed::RustEmbed;
    pub use serde::{Deserialize, Serialize};
}

/// A prelude for Wintray applications, re-exporting commonly used types.
pub mod prelude {
    pub use super::exports::*;
}

pub use engine::{WintrayApp, WintrayAppBuilder};
pub use wintray_macros::{wintray_assets, wintray_template};
