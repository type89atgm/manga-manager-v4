use crate::config::AppConfig;
use crate::db::Database;
use crate::eh::{client::EhClient, batch, retry};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize)]
pub struct FetchResult {
    pub success: Vec<i64>,
    pub failed: Vec<i64>,
    pub matched: Vec<i64>,
    pub ip_banned: bool,
}

/// Fetch tags by gallery URL
#[tauri::command]
pub async fn fetch_tags_by_url(
    manga_id: i64,
    url: String,
    db: State<'_, Mutex<Database>>,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<(), String> {
    let (use_exh, cookies) = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        (cfg.use_exhentai, EhClient::build_cookie_string(&cfg))
    };
    let re = regex::Regex::new(r"/g/(\d+)/([a-f0-9]+)").map_err(|e| e.to_string())?;
    let caps = re.captures(&url).ok_or("无效的画廊链接")?;
    let gid: i64 = caps[1].parse().map_err(|e| e.to_string())?;
    let token = caps[2].to_string();

    let gidlist = vec![(gid, token.clone())];
    let result = retry::retry_with_backoff(|| {
        let c = EhClient::new_from(use_exh);
        let g = gidlist.clone();
        let ck = cookies.clone();
        async move { c.fetch_gdata(&g, &ck).await }
    }).await?;

    if result.is_empty() { return Err("未获取到元数据".into()); }
    let meta = &result[0];
    let mut db = db.lock().map_err(|e| e.to_string())?;
    // Also save GID+token
    db.conn.execute("UPDATE manga SET eh_gid=?1, eh_token=?2 WHERE id=?3",
        rusqlite::params![gid.to_string(), token, manga_id]).map_err(|e| e.to_string())?;
    save_metadata(&mut db, manga_id, meta)?;
    Ok(())
}

/// Match manga to ExHentai galleries by SHA1 hash
#[tauri::command]
pub async fn match_by_sha1(
    manga_ids: Vec<i64>,
    db: State<'_, Mutex<Database>>,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<Vec<i64>, String> {
    let (use_exh, cookies) = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        (cfg.use_exhentai, EhClient::build_cookie_string(&cfg))
    };

    let client = EhClient::new_from(use_exh);
    let mut matched = Vec::new();

    for mid in manga_ids {
        let sha1: Option<String> = {
            let db2 = db.lock().map_err(|e| e.to_string())?;
            db2.conn.query_row(
                "SELECT sha1_hash FROM manga WHERE id=?1",
                rusqlite::params![mid],
                |r| r.get(0)
            ).ok()
        };

        if let Some(hash) = sha1 {
            if hash.is_empty() { continue; }
            match client.search_by_sha1(&hash, &cookies).await {
                Ok(Some((gid, token))) => {
                    let mut db3 = db.lock().map_err(|e| e.to_string())?;
                    db3.conn.execute(
                        "UPDATE manga SET eh_gid=?1, eh_token=?2 WHERE id=?3",
                        rusqlite::params![gid, token, mid]
                    ).map_err(|e| e.to_string())?;
                    matched.push(mid);
                }
                _ => {}
            }
        }
    }
    Ok(matched)
}

/// Batch fetch: first match by SHA1, then fetch gdata
#[tauri::command]
pub async fn batch_fetch_tags(
    db: State<'_, Mutex<Database>>,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<FetchResult, String> {
    let (use_exh, cookies) = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        (cfg.use_exhentai, EhClient::build_cookie_string(&cfg))
    };

    // Step 1: Find untagged manga with SHA1 but no GID
    let needs_match: Vec<i64> = {
        let db2 = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db2.conn.prepare(
            "SELECT id FROM manga WHERE tag_status IN ('non-tag','tag-failed') AND (eh_gid IS NULL OR eh_gid='') AND sha1_hash IS NOT NULL AND sha1_hash!=''"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map([], |r| r.get::<_, i64>(0)).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        x
    };

    // Step 2: Match by SHA1
    let mut matched = Vec::new();
    if !needs_match.is_empty() {
        let client = EhClient::new_from(use_exh);
        for mid in &needs_match {
            let sha1: Option<String> = {
                let db3 = db.lock().map_err(|e| e.to_string())?;
                db3.conn.query_row(
                    "SELECT sha1_hash FROM manga WHERE id=?1",
                    rusqlite::params![mid],
                    |r| r.get(0)
                ).ok()
            };
            if let Some(hash) = sha1 {
                if hash.is_empty() { continue; }
                match client.search_by_sha1(&hash, &cookies).await {
                    Ok(Some((gid, token))) => {
                        let mut db4 = db.lock().map_err(|e| e.to_string())?;
                        let _ = db4.conn.execute(
                            "UPDATE manga SET eh_gid=?1, eh_token=?2 WHERE id=?3",
                            rusqlite::params![gid, token, mid]
                        );
                        matched.push(*mid);
                    }
                    _ => {}
                }
            }
        }
    }

    // Step 3: Fetch gdata for all manga with GID
    let rows: Vec<(i64, i64, String)> = {
        let db5 = db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db5.conn.prepare(
            "SELECT id, eh_gid, eh_token FROM manga WHERE tag_status IN ('non-tag','tag-failed') AND eh_gid IS NOT NULL AND eh_gid!=''"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map([], |r| Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?.parse().unwrap_or(0),
            r.get::<_, String>(2)?,
        ))).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        x
    };

    if rows.is_empty() {
        return Ok(FetchResult { success: vec![], failed: vec![], matched, ip_banned: false });
    }

    let gidlist: Vec<(i64, String)> = rows.iter().map(|(_, g, t)| (*g, t.clone())).collect();
    let client = EhClient::new_from(use_exh);
    let results = batch::batch_fetch_with(&client, &gidlist, &cookies).await;

    let mut dbf = db.lock().map_err(|e| e.to_string())?;
    match results {
        Ok(metas) => {
            let mut success = Vec::new();
            let mut failed = Vec::new();
            for (i, meta) in metas.iter().enumerate() {
                if i < rows.len() {
                    let mid = rows[i].0;
                    match save_metadata(&mut dbf, mid, meta) {
                        Ok(_) => success.push(mid),
                        Err(_) => { failed.push(mid); mark_failed(&mut dbf, mid); }
                    }
                }
            }
            Ok(FetchResult { success, failed, matched, ip_banned: false })
        }
        Err(e) if e == "IP_BANNED" => {
            for (id, _, _) in &rows { mark_failed(&mut dbf, *id); }
            Ok(FetchResult { success: vec![], failed: rows.iter().map(|r| r.0).collect(), matched, ip_banned: true })
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn retry_failed_tags(
    db: State<'_, Mutex<Database>>,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<FetchResult, String> {
    batch_fetch_tags(db, config).await
}

#[tauri::command]
pub fn set_eh_cookies(
    ipb_member_id: String,
    ipb_pass_hash: String,
    igneous: String,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    config.eh_cookies.ipb_member_id = ipb_member_id;
    config.eh_cookies.ipb_pass_hash = ipb_pass_hash;
    config.eh_cookies.igneous = igneous;
    let data_dir = dirs::data_dir().ok_or("no data dir")?.join("manga-manager-v4");
    config.save(&data_dir);
    Ok(())
}

#[tauri::command]
pub async fn search_eh_gallery(
    keyword: String,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<serde_json::Value, String> {
    let (use_exh, cookies) = {
        let cfg = config.lock().map_err(|e| e.to_string())?;
        (cfg.use_exhentai, EhClient::build_cookie_string(&cfg))
    };
    let client = EhClient::new_from(use_exh);
    client.search(&keyword, 0, &cookies).await
}

fn save_metadata(db: &mut Database, manga_id: i64, meta: &crate::db::models::EhGalleryMetadata) -> Result<(), String> {
    db.conn.execute(
        "UPDATE manga SET eh_gid=?1, eh_token=?2, eh_title=?3, eh_title_jpn=?4, eh_category=?5, \
         eh_posted=?6, eh_uploader=?7, eh_filesize=?8, eh_thumb_url=?9, \
         tag_status='tagged', updated_at=datetime('now') WHERE id=?10",
        rusqlite::params![
            meta.gid.to_string(), meta.token, meta.title, meta.title_jpn,
            meta.category, meta.posted, meta.uploader, meta.filesize, meta.thumb,
            manga_id
        ],
    ).map_err(|e| e.to_string())?;

    for tag_str in &meta.tags {
        let parts: Vec<&str> = tag_str.splitn(2, ':').collect();
        let (ns, t) = if parts.len() == 2 { (parts[0], parts[1]) } else { ("misc", parts[0]) };
        db.conn.execute("INSERT OR IGNORE INTO tags (namespace,tag) VALUES (?1,?2)", rusqlite::params![ns, t]).map_err(|e| e.to_string())?;
        let tag_id: i64 = db.conn.query_row("SELECT id FROM tags WHERE namespace=?1 AND tag=?2", rusqlite::params![ns, t], |r| r.get(0)).map_err(|e| e.to_string())?;
        db.conn.execute("INSERT OR IGNORE INTO manga_tags (manga_id,tag_id) VALUES (?1,?2)", rusqlite::params![manga_id, tag_id]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn mark_failed(db: &mut Database, manga_id: i64) {
    let _ = db.conn.execute("UPDATE manga SET tag_status='tag-failed' WHERE id=?1", rusqlite::params![manga_id]);
}