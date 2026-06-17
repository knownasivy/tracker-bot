use axum::{Router, routing::get};

use crate::{
    frontend::handlers::{about_page, file_page, home_page},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .route("/index.html", get(home_page))
        .route("/about", get(about_page))
        .route("/u/{upload_id}", get(file_page))
}
