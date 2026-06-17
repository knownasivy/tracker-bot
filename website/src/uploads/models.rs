use std::path::Path;

use nanoid::nanoid;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileBlob {
    pub id: Uuid,
    pub file_path: String,
    pub hash: Vec<u8>, // Blake3 hash
    pub size: i64,
    /* TODO: Store file format
        make sure file format matches file extension of File.
        File format will be detected by magic string.
    */
}

// TODO: Remove default
#[derive(Debug, Clone)]
pub struct FileUpload {
    pub id: Uuid,
    pub upload_id: String, // Sharable id
    pub blob_id: Uuid,
    pub original_name: String,
    pub created_at: OffsetDateTime,
}

impl FileUpload {
    pub fn extension(&self) -> Option<&str> {
        Path::new(&self.original_name)
            .extension()
            .and_then(|e| e.to_str())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileResponse {
    pub upload_id: String,
}

impl From<FileUpload> for FileResponse {
    fn from(file: FileUpload) -> Self {
        Self {
            upload_id: file.upload_id,
        }
    }
}
