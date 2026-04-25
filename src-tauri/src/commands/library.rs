use crate::archive;
use crate::cache::thumbnail::{compute_hash, generate_thumbnail};
use crate::cache::ThumbnailCache;
use crate::config::AppConfig;
use crate::db::{Database, models::MangaListItem};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize)]
pub struct ScanResult {
    pub added: i64,
    pub total: i64,
}

fn compute_sha1(data: &[u8]) -> String {
    use sha2::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn try_read_ehviewer(file_path: &str) -> Option<(String, String)> {
    let p = std::path::Path::new(file_path);
    let dir = p.parent()?;
    let eh_path = dir.join(".ehviewer");
    if eh_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&eh_path) {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() >= 4 {
                let gid = lines[2].trim().to_string();
                let token = lines[3].trim().to_string();
                if !gid.is_empty() && !token.is_empty() {
                    return Some((gid, token));
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn scan_library(
    paths: Vec<String>,
    db: State<'_, Mutex<Database>>,
    config: State<'_, Mutex<AppConfig>>,
    cache: State<'_, ThumbnailCache>,
) -> Result<ScanResult, String> {
    let mut db = db.lock().map_err(|e| e.to_string())?;
    let mut config = config.lock().map_err(|e| e.to_string())?;
    config.library_paths = paths.clone();
    let data_dir = dirs::data_dir().ok_or("no data dir")?.join("manga-manager-v4");
    config.save(&data_dir);

    let mut added = 0i64;
    let mut total = 0i64;
    for lib_path in &paths {
        if !std::path::Path::new(lib_path).exists() { continue; }
        scan_dir(lib_path, &mut db, &cache, &mut added, &mut total)?;
    }
    Ok(ScanResult { added, total })
}

fn scan_dir(
    dir: &str,
    db: &mut Database,
    cache: &ThumbnailCache,
    added: &mut i64,
    total: &mut i64,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path.to_string_lossy(), db, cache, added, total)?;
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !matches!(ext.as_str(), "zip" | "cbz" | "rar" | "cbr") { continue; }
        *total += 1;
        let file_path = path.to_string_lossy().to_string();

        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM manga WHERE file_path=?1",
            rusqlite::params![file_path],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        if count > 0 { continue; }

        let file_bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let file_hash = compute_hash(&file_bytes);
        let sha1_hash = compute_sha1(&file_bytes);
        let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
        let folder = path.parent().and_then(|p| p.to_str()).unwrap_or("").to_string();
        let images = archive::list_images(&path)?;
        let page_count = images.len() as i64;

        let cover_cache_path = if !images.is_empty() {
            archive::read_image(&path, &images[0].name)
                .ok()
                .and_then(|d| generate_thumbnail(&d, 300, &cache.dir, &file_hash).ok())
        } else { None };

        // Check for .ehviewer file
        let (eh_gid, eh_token) = try_read_ehviewer(&file_path).unwrap_or((String::new(), String::new()));
        let tag_status = if eh_gid.is_empty() { "non-tag" } else { "non-tag" };

        db.conn.execute(
            "INSERT INTO manga (title,file_path,file_hash,sha1_hash,folder,page_count,cover_cache_path,eh_gid,eh_token,tag_status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            rusqlite::params![title, file_path, file_hash, sha1_hash, folder, page_count, cover_cache_path, eh_gid, eh_token, tag_status],
        ).map_err(|e| e.to_string())?;
        *added += 1;
    }
    Ok(())
}

#[tauri::command]
pub fn list_manga(
    folder: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<MangaListItem>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let like = folder.map(|f| format!("{}%", f)).unwrap_or_else(|| "%".to_string());
    let mut stmt = db.conn.prepare(
        "SELECT id,title,folder,page_count,score,cover_cache_path,tag_status,eh_category FROM manga WHERE folder LIKE ?1 ORDER BY title"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(
        rusqlite::params![like],
        |r| Ok(MangaListItem {
            id: r.get(0)?, title: r.get(1)?, folder: r.get(2)?, page_count: r.get(3)?,
            score: r.get(4)?, cover_cache_path: r.get(5)?, tag_status: r.get(6)?, eh_category: r.get(7)?,
        }),
    ).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn search_manga(
    keyword: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<MangaListItem>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", keyword);
    let mut stmt = db.conn.prepare(
        "SELECT m.id,m.title,m.folder,m.page_count,m.score,m.cover_cache_path,m.tag_status,m.eh_category \
         FROM manga m LEFT JOIN manga_tags mt ON m.id=mt.manga_id LEFT JOIN tags t ON mt.tag_id=t.id \
         WHERE m.title LIKE ?1 OR t.tag LIKE ?1 GROUP BY m.id ORDER BY m.title"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(
        rusqlite::params![pattern],
        |r| Ok(MangaListItem {
            id: r.get(0)?, title: r.get(1)?, folder: r.get(2)?, page_count: r.get(3)?,
            score: r.get(4)?, cover_cache_path: r.get(5)?, tag_status: r.get(6)?, eh_category: r.get(7)?,
        }),
    ).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}