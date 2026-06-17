use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{
    state::AppState,
    uploads::handlers::{download_file, upload_file},
};

const MAX_SIZE: usize = 200 * 1000 * 1000;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/uploads",
            post(upload_file).layer(DefaultBodyLimit::max(MAX_SIZE)), // 200MB max upload
        )
        .route("/uploads/{upload_id}/download", get(download_file))
}
