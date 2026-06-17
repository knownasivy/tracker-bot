use sqlx::{PgPool, postgres::PgPoolOptions};

use sqlx::migrate::Migrator;

// static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn create_pool(url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(2)
        .connect(url)
        .await?;

//    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
