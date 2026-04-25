use std::path::PathBuf;

pub fn cache_image(data: &[u8], cache_dir: &PathBuf, manga_id: i64, page: usize, ext: &str) -> Result<String, String> {
    let dir = cache_dir.join(manga_id.to_string());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.{}", page, ext));
    if !path.exists() {
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().to_string())
}

#[allow(dead_code)]
pub fn clear_cache(cache_dir: &PathBuf, manga_id: i64) -> Result<(), String> {
    let dir = cache_dir.join(manga_id.to_string());
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}