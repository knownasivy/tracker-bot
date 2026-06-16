use axum::{
    extract::multipart::MultipartError, http::StatusCode, response::{IntoResponse, Response}
};

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Database(sqlx::Error),

    // TODO: Make error more specific when doing IntoResponse
    Multipart(MultipartError),
    Io(std::io::Error),

    NoFile, // No File in upload
}

// TODO: Json errors?
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            ApiError::Database(err) => {
                tracing::error!("Database error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
            ApiError::Multipart(err) => {
                tracing::error!("Multipart error: {err}");
                err.into_response()
            }
            ApiError::Io(err) => {
                tracing::error!("Io Error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Error while uploading file").into_response()
            }

            ApiError::NoFile => {
                (StatusCode::NOT_FOUND, "No file in multipart request").into_response()
            }
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound,
            err => ApiError::Database(err),
        }
    }
}

impl From<MultipartError> for ApiError {
    fn from(err: MultipartError) -> Self {
        ApiError::Multipart(err)
    }
}


impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Io(err)
    }
}
