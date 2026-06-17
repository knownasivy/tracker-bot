use crate::{
    error::ApiError,
    state::AppState,
    uploads::{models::FileResponse, queries},
};
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{
        HeaderValue, Request,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::Response,
};
use blake3::Hasher;
use std::path;
use time::OffsetDateTime;
use tokio::fs;
use tokio::io::{self, AsyncWriteExt};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use uuid::Uuid;

// pub async fn get_file(
//     State(state): State<AppState>,
//     Path(id): Path<Uuid>,
// ) -> Result<Json<FileResponse>, ApiError> {
//     let file = queries::find_file_upload(&state.db, id).await?;

//     if let Some(f) = file {
//         return Ok(Json(f.into()));
//     }

//     Err(ApiError::NotFound)
// }

pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileResponse>, ApiError> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let file_name = sanitize_filename::sanitize(field.file_name().unwrap_or("file.bin"));

        let temp_file = state.temp_dir.join(Uuid::now_v7().to_string());
        let mut file = fs::File::create(temp_file.clone()).await?;

        // TODO: Put in tokio io blocking code block?
        // TODO: Check magic, check file ext, check mime-type

        let mut hasher = Hasher::new();
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
        }

        file.flush().await?;

        let hash = *hasher.finalize().as_bytes();
        let size = fs::metadata(temp_file.clone()).await?.len() as i64;

        let now = OffsetDateTime::now_utc();

        let date = format!(
            "{year}/{month:02}-{day:02}",
            year = now.year(),
            month = now.month() as u8,
            day = now.day()
        );

        // TODO: Store full path from the relative path.
        let new_path = format!(
            "{path}/{date}/{uuid}",
            path = state.upload_path,
            uuid = Uuid::now_v7()
        );

        let (blob_id, inserted) =
            queries::insert_file_blob(&state.db, &new_path, &hash, size).await?;

        if inserted {
            move_file(temp_file, new_path.clone()).await?;

            tracing::info!("new file: {}", new_path);
        } else {
            fs::remove_file(temp_file).await?;
        }

        let file = queries::insert_file_upload(&state.db, blob_id, &file_name, size).await?;

        return Ok(Json(file.into()));
    }

    Err(ApiError::NoFile)
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(upload_id): Path<String>,
    request: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    let Some(file) = queries::find_file_upload_by_short_code(&state.db, &upload_id).await? else {
        return Err(ApiError::NotFound);
    };

    let blob = queries::find_file_blob(&state.db, file.blob_id).await?;

    let mut response = ServeFile::new(&blob.file_path)
        .oneshot(request)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve file: {e}"))?
        .map(axum::body::Body::new);

    response.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename*=UTF-8''{}",
            urlencoding::encode(&file.original_name)
        ))
        .unwrap(),
    );

    // Optional if you want browser sniffing instead.
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    Ok(response)
}

// TODO: preview
// pub async fn download_file(
//     State(state): State<AppState>,
//     Path(upload_id): Path<String>,
//     request: Request<Body>,
// ) -> Result<Response<Body>, ApiError> {
//     let Some(file) = queries::find_file_upload_by_short_code(&state.db, &upload_id).await? else {
//         return Err(ApiError::NotFound);
//     };

//     let blob = queries::find_file_blob(&state.db, file.blob_id).await?;

//     let mut response = ServeFile::new(&blob.file_path)
//         .oneshot(request)
//         .await
//         .map_err(|e| anyhow::anyhow!("Failed to serve file: {e}"))?
//         .map(axum::body::Body::new);

//     response.headers_mut().insert(
//         CONTENT_DISPOSITION,
//         HeaderValue::from_str(&format!(
//             "attachment; filename*=UTF-8''{}",
//             urlencoding::encode(&file.original_name)
//         ))
//         .unwrap(),
//     );

//     // Optional if you want browser sniffing instead.
//     response.headers_mut().insert(
//         CONTENT_TYPE,
//         HeaderValue::from_static("application/octet-stream"),
//     );

//     Ok(response)
// }

pub async fn move_file<P: AsRef<path::Path>, Q: AsRef<path::Path>>(
    from: P,
    to: Q,
) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if fs::rename(from, to).await.is_ok() {
        return Ok(());
    }

    fs::create_dir_all(
        to.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no parent dir"))?,
    )
    .await?;

    fs::copy(from, to).await?;

    fs::remove_file(from).await?;

    Ok(())
}
