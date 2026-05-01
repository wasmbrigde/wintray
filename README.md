# Wintray

`wintray` is a lightweight Rust framework for building Windows system tray applications with an embedded web-based user interface. It combines a tray icon, custom context menus, and a secure Axum-powered web server into a single, easy-to-use package.

## Features

*   **System Tray Integration**: Easily create tray icons with tooltips and custom context menus.
*   **Embedded Web Server**: Built-in [Axum](https://github.com/tokio-rs/axum) server for your application's UI.
*   **Automatic TLS**: Automatically generates and manages self-signed certificates for secure HTTPS communication locally.
*   **Asset Embedding**: Utilities for serving static web assets embedded directly in the binary.
*   **Simple Configuration**: Helpers for loading and saving YAML-based configurations.

## Getting Started

### 1. Define your application

Use the `WintrayAppBuilder` to configure your application:

```rust
use wintray::WintrayAppBuilder;
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // Define your web UI routes
    let router = Router::new()
        .route("/", get(|| async { "Hello from Wintray!" }));

    // Build the app
    let app = WintrayAppBuilder::new()
        .with_tooltip("My Awesome App")
        .with_icon(include_bytes!("../assets/icon.svg")) // Pass SVG bytes
        .with_router(router)
        .with_address("127.0.0.1:9876")
        .add_menu_item("settings", "Settings")
        .build();

    // Run the app with a custom menu handler
    app.run_with(|menu_id| {
        match menu_id {
            "settings" => println!("Settings clicked!"),
            _ => {}
        }
    });
}
```

### 2. Interaction

*   **Left-click on tray icon**: Automatically opens the web UI in the default browser using the configured address.
*   **"Open UI" menu item**: Standard menu item to open the web UI.
*   **"Quit" menu item**: Standard menu item to exit the application.
*   **Custom Menu Items**: Any items added via `.add_menu_item()` will trigger the closure passed to `run_with`.

## Components

*   **`WintrayAppBuilder`**: Fluent API for constructing your application.
*   **`config` module**: Provides `load_config` and `save_config` for persistent YAML settings.
*   **`assets` module**: Utilities to serve static files embedded with `rust-embed`.

## Platform Support

Currently, `wintray` only supports **Windows**, as it relies on Windows-specific tray icon behaviors and path conventions.
