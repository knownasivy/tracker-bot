use sqlx::PgPool;
use anyhow::Result;
use crate::db::postgres;
use tokio::fs;

#[derive(Clone)]
pub struct AppState {
    pub upload_path: &'static str,
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
            db: postgres::create_pool(&url).await?,
        };
        
        Ok(new)
    }
}