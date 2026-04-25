use crate::archive;
use crate::cache::image_cache::cache_image;
use crate::cache::ImageCache;
use crate::db::Database;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize)]
pub struct PageInfo {
    pub index: usize,
    pub name: String,
}

#[tauri::command]
pub fn list_pages(
    manga_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<PageInfo>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let file_path: String = db.conn.query_row(
        "SELECT file_path FROM manga WHERE id=?1",
        rusqlite::params![manga_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    let images = archive::list_images(std::path::Path::new(&file_path))?;
    Ok(images.into_iter().map(|e| PageInfo { index: e.index, name: e.name }).collect())
}

#[tauri::command]
pub fn get_page(
    manga_id: i64,
    page: usize,
    db: State<'_, Mutex<Database>>,
    img_cache: State<'_, ImageCache>,
) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let file_path: String = db.conn.query_row(
        "SELECT file_path FROM manga WHERE id=?1",
        rusqlite::params![manga_id],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    let path = std::path::Path::new(&file_path);
    let images = archive::list_images(path)?;
    if page >= images.len() {
        return Err("page out of range".into());
    }
    let entry_name = &images[page].name;
    let img_data = archive::read_image(path, entry_name)?;
    let ext = std::path::Path::new(entry_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    cache_image(&img_data, &img_cache.dir, manga_id, page, ext)
}