use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FilebinResponse {
    pub bin: BinInfo,
    pub file: FileInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinInfo {
    pub id: String,
    pub readonly: bool,
    pub bytes: u64,
    #[serde(rename = "bytes_readable")]
    pub bytes_readable: String,
    pub files: u64,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "updated_at_relative")]
    pub updated_at_relative: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "created_at_relative")]
    pub created_at_relative: String,
    #[serde(rename = "expired_at")]
    pub expired_at: String,
    #[serde(rename = "expired_at_relative")]
    pub expired_at_relative: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    #[serde(rename = "content-type")]
    pub content_type: String,
    pub bytes: u64,
    #[serde(rename = "bytes_readable")]
    pub bytes_readable: String,
    pub md5: String,
    pub sha256: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "updated_at_relative")]
    pub updated_at_relative: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "created_at_relative")]
    pub created_at_relative: String,
}
