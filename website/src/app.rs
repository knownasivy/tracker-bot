use axum::Router;
use tower_http::services::ServeDir;

use crate::{
    frontend,
    state::AppState,
    uploads::{self},
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(frontend::routes::router())
        .nest("/api", uploads::routes::router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
