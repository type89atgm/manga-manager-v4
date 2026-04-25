use crate::db::{Database, models::{MangaDetail, Tag}};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn set_score(
    manga_id: i64,
    score: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let score = score.clamp(0.0, 10.0);
    db.conn.execute(
        "UPDATE manga SET score=?1, updated_at=datetime('now') WHERE id=?2",
        rusqlite::params![score, manga_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_manga_detail(
    manga_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<MangaDetail, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let manga = db.conn.query_row(
        "SELECT * FROM manga WHERE id=?1",
        rusqlite::params![manga_id],
        |r| {
            Ok(crate::db::models::Manga {
                id: r.get(0)?,
                title: r.get(1)?,
                file_path: r.get(2)?,
                file_hash: r.get(3)?,
                folder: r.get(4)?,
                page_count: r.get(5)?,
                score: r.get(6)?,
                cover_cache_path: r.get(7)?,
                eh_gid: r.get(8)?,
                eh_token: r.get(9)?,
                eh_title: r.get(10)?,
                eh_title_jpn: r.get(11)?,
                eh_category: r.get(12)?,
                eh_posted: r.get(13)?,
                eh_uploader: r.get(14)?,
                eh_filesize: r.get(15)?,
                eh_thumb_url: r.get(16)?,
                tag_status: r.get(17)?,
                created_at: r.get(18)?,
                updated_at: r.get(19)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    let mut stmt = db.conn.prepare(
        "SELECT t.id, t.namespace, t.tag FROM tags t \
         JOIN manga_tags mt ON t.id=mt.tag_id WHERE mt.manga_id=?1"
    ).map_err(|e| e.to_string())?;
    let tags: Vec<Tag> = stmt.query_map(
        rusqlite::params![manga_id],
        |r| {
            Ok(Tag {
                id: r.get(0)?,
                namespace: r.get(1)?,
                tag: r.get(2)?,
            })
        },
    ).map_err(|e| e.to_string())?.filter_map(|t| t.ok()).collect();

    let read_progress: Option<i64> = db.conn.query_row(
        "SELECT current_page FROM read_progress WHERE manga_id=?1",
        rusqlite::params![manga_id],
        |r| r.get(0),
    ).ok();

    Ok(MangaDetail { manga, tags, read_progress })
}

#[tauri::command]
pub fn update_manga_tags(
    manga_id: i64,
    tags: Vec<(String, String)>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn.execute("DELETE FROM manga_tags WHERE manga_id=?1", rusqlite::params![manga_id])
        .map_err(|e| e.to_string())?;
    for (ns, tag) in tags {
        db.conn.execute(
            "INSERT OR IGNORE INTO tags (namespace,tag) VALUES (?1,?2)",
            rusqlite::params![ns, tag],
        ).map_err(|e| e.to_string())?;
        let tag_id: i64 = db.conn.query_row(
            "SELECT id FROM tags WHERE namespace=?1 AND tag=?2",
            rusqlite::params![ns, tag],
            |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        db.conn.execute(
            "INSERT OR IGNORE INTO manga_tags (manga_id,tag_id) VALUES (?1,?2)",
            rusqlite::params![manga_id, tag_id],
        ).map_err(|e| e.to_string())?;
    }
    db.conn.execute(
        "UPDATE manga SET tag_status='tagged', updated_at=datetime('now') WHERE id=?1",
        rusqlite::params![manga_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}