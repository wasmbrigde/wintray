use axum::{
    body::Body,
    extract::Path,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

/// Embedded static assets for the framework UI.
#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct FrameworkAssets;

/// Axum handler that serves embedded static assets from the `assets/` directory.
pub async fn framework_assets_handler(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    match FrameworkAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Asset Not Found"))
            .unwrap(),
    }
}
