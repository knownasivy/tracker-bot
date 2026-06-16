use axum::Router;

use crate::{
    state::AppState,
    uploads::{self},
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", uploads::routes::router())
        .with_state(state)
}
