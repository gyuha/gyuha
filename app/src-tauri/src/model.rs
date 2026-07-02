use serde::Serialize;

/// A single file-system entry shown in a pane.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    /// Modification time as epoch milliseconds (None if unavailable).
    pub modified: Option<i64>,
    pub extension: String,
}

/// The result of listing a directory.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DirListing {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
}

/// A mounted drive / volume.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub name: String,
    pub path: String,
    pub total: u64,
    pub available: u64,
}
