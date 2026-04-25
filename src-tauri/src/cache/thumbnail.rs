use image::ImageFormat;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub fn generate_thumbnail(data: &[u8], size: u32, cache_dir: &PathBuf, file_hash: &str) -> Result<String, String> {
    let cache_path = cache_dir.join(format!("{}.webp", file_hash));
    if cache_path.exists() {
        return Ok(cache_path.to_string_lossy().to_string());
    }
    let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
    let thumb = img.thumbnail(size, size);
    thumb.save_with_format(&cache_path, ImageFormat::WebP).map_err(|e| e.to_string())?;
    Ok(cache_path.to_string_lossy().to_string())
}

pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())[..16].to_string()
}