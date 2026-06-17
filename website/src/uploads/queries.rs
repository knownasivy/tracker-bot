use nanoid::nanoid;
use sqlx::PgPool;
use uuid::Uuid;

use crate::uploads::models::{FileBlob, FileUpload};

pub async fn find_file_upload(db: &PgPool, id: Uuid) -> Result<Option<FileUpload>, sqlx::Error> {
    let file = sqlx::query_as!(
        FileUpload,
        r#"
        SELECT *
        FROM files
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(file)
}

pub async fn insert_file_upload(
    pool: &PgPool,
    blob_id: Uuid,
    original_name: &str,
) -> anyhow::Result<FileUpload> {
    let blob = sqlx::query_as!(
        FileUpload,
        r#"
        INSERT INTO files (id, upload_id, blob_id, original_name)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        Uuid::now_v7(),
        nanoid!(8),
        blob_id,
        original_name
    )
    .fetch_one(pool)
    .await?;

    Ok(blob)
}

pub async fn find_file_blob(db: &PgPool, id: Uuid) -> anyhow::Result<Option<FileBlob>> {
    let blob = sqlx::query_as!(
        FileBlob,
        r#"
        SELECT *
        FROM file_blobs
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(blob)
}

pub async fn find_file_blob_by_hash(
    pool: &PgPool,
    hash: &[u8; 32],
) -> Result<Option<FileBlob>, sqlx::Error> {
    let blob = sqlx::query_as!(
        FileBlob,
        r#"
        SELECT *
        FROM file_blobs
        WHERE hash = $1
        "#,
        hash.as_slice()
    )
    .fetch_optional(pool)
    .await?;

    Ok(blob)
}

pub async fn insert_file_blob(
    pool: &PgPool,
    path: &str,
    hash: &[u8; 32],
    size: i64,
) -> anyhow::Result<(Uuid, bool)> {
    let id = Uuid::now_v7();

    let row = sqlx::query!(
        r#"
        INSERT INTO file_blobs (id, file_path, hash, size)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (hash)
        DO UPDATE SET hash = file_blobs.hash
        RETURNING id, (xmax = 0) AS inserted
        "#,
        id,
        path,
        hash,
        size
    )
    .fetch_one(pool)
    .await?;

    Ok((row.id, row.inserted.unwrap_or(false)))
}
