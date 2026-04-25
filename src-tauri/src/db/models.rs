use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manga {
    pub id: i64,
    pub title: String,
    pub file_path: String,
    pub file_hash: String,
    pub sha1_hash: Option<String>,
    pub folder: String,
    pub page_count: i64,
    pub score: f64,
    pub cover_cache_path: Option<String>,
    pub eh_gid: Option<String>,
    pub eh_token: Option<String>,
    pub eh_title: Option<String>,
    pub eh_title_jpn: Option<String>,
    pub eh_category: Option<String>,
    pub eh_posted: Option<String>,
    pub eh_uploader: Option<String>,
    pub eh_filesize: Option<i64>,
    pub eh_thumb_url: Option<String>,
    pub tag_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub namespace: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDetail {
    pub manga: Manga,
    pub tags: Vec<Tag>,
    pub read_progress: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaListItem {
    pub id: i64,
    pub title: String,
    pub folder: String,
    pub page_count: i64,
    pub score: f64,
    pub cover_cache_path: Option<String>,
    pub tag_status: String,
    pub eh_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EhGalleryMetadata {
    pub gid: i64,
    pub token: String,
    pub title: String,
    pub title_jpn: Option<String>,
    pub category: String,
    pub thumb: String,
    pub uploader: String,
    pub posted: String,
    pub filecount: String,
    pub filesize: i64,
    pub tags: Vec<String>,
}