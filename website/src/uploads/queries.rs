use sqlx::{PgPool};
use uuid::Uuid;

use crate::uploads::models::File;

pub async fn find_file(
    db: &PgPool,
    id: Uuid,
) -> Result<File, sqlx::Error> {
    let file = sqlx::query_as!(
        File,
        "SELECT id, name FROM files WHERE id = $1",
        id
    )
    .bind(id)
    .fetch_one(db)
    .await?;

    Ok(file)
}