use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{
    state::AppState,
    uploads::handlers::{get_file, upload_file},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/uploads",
            post(upload_file).layer(DefaultBodyLimit::max(200 * 1000 * 1000)), // 200MB max upload
        )
        .route("/uploads/{id}", get(get_file))
}
