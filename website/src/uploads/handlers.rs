use crate::{
    error::ApiError,
    state::AppState,
    uploads::{models::FileResponse, queries},
};
use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use std::path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FileResponse>, ApiError> {
    let file = queries::find_file(&state.db, id).await?;

    Ok(Json(file.into()))
}

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileResponse>, ApiError> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let file_name = sanitize_filename::sanitize(field.file_name().unwrap_or("file.bin"));

        let id = Uuid::now_v7();
        let ext = extract_ext(&file_name).unwrap_or(".bin".into());

        // TODO: Insert into db

        let path = format!("./uploads/{id}.{ext}");
        let mut file = fs::File::create(&path).await?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }
    }

    Err(ApiError::NoFile)
}

fn extract_ext(name: &str) -> Option<String> {
    path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}
