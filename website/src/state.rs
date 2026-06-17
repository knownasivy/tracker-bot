use std::path::PathBuf;

use crate::db::postgres;
use anyhow::Result;
use sqlx::PgPool;
use tokio::fs;

#[derive(Clone)]
pub struct AppState {
    pub upload_path: &'static str,
    pub temp_dir: PathBuf,
    pub db: PgPool,
    // pub redis: RedisPool,
    // Arc<Config> for typed conf ?
}

// TODO: move to a config
const UPLOAD_PATH: &str = "./uploads";

impl AppState {
    pub async fn new() -> Result<Self> {
        let url = std::env::var("DATABASE_URL").expect("No 'DATABASE_URL' env var found.");

        fs::create_dir_all(UPLOAD_PATH).await?;

        let new = Self {
            upload_path: UPLOAD_PATH,
            temp_dir: std::env::temp_dir(),
            db: postgres::create_pool(&url).await?,
        };

        Ok(new)
    }
}
