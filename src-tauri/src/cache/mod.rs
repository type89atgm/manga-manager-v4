pub mod thumbnail;
pub mod image_cache;

use std::path::PathBuf;

pub struct ThumbnailCache {
    pub dir: PathBuf,
}

impl ThumbnailCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

pub struct ImageCache {
    pub dir: PathBuf,
}

impl ImageCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}