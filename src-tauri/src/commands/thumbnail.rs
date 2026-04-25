use crate::archive;
use crate::cache::thumbnail::{compute_hash, generate_thumbnail};
use crate::cache::ThumbnailCache;
use crate::db::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_thumbnail(
    manga_id: i64,
    db: State<'_, Mutex<Database>>,
    cache: State<'_, ThumbnailCache>,
) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let (file_path, cover_cache): (String, Option<String>) = db.conn.query_row(
        "SELECT file_path, cover_cache_path FROM manga WHERE id=?1",
        rusqlite::params![manga_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| e.to_string())?;

    if let Some(p) = cover_cache {
        if std::path::Path::new(&p).exists() {
            return Ok(p);
        }
    }

    let path = std::path::Path::new(&file_path);
    let images = archive::list_images(path)?;
    if images.is_empty() {
        return Err("no images in archive".into());
    }
    let img_data = archive::read_image(path, &images[0].name)?;
    let file_hash = compute_hash(&std::fs::read(path).map_err(|e| e.to_string())?);
    generate_thumbnail(&img_data, 300, &cache.dir, &file_hash)
}