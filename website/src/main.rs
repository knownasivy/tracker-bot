use dotenvy::dotenv;
use std::net::SocketAddr;

pub mod app;
pub mod db;
pub mod error;
pub mod frontend;
pub mod state;
pub mod uploads;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();

    // build our application with a single route
    let state = state::AppState::new().await?;
    let app = app::build_router(state);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Listening on: {}", addr);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
