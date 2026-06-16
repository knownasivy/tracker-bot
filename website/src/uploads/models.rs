use uuid::Uuid;

// TODO: Remove default
#[derive(Clone, serde::Serialize)]
pub struct File {
    pub id: Uuid,
    pub name: String,
    // file path?
    // create date?
    // share string?
    // type?
}

impl From<File> for FileResponse {
    fn from(file: File) -> Self {
        Self {
            id: file.id,
            name: file.name,
        }
    }
}

#[derive(Clone, serde::Serialize, Default)]
pub struct FileResponse {
    pub id: Uuid,
    pub name: String,
}
