mod archive;
mod cache;
mod commands;
mod config;
mod db;
mod eh;

use commands::*;
use db::Database;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");
            let db_path = data_dir.join("manga.db");
            let db = Database::new(&db_path).expect("failed to init database");
            app.manage(Mutex::new(db));
            let config = config::AppConfig::load(&data_dir);
            app.manage(Mutex::new(config));
            let cache_dir = data_dir.join("thumbnails");
            std::fs::create_dir_all(&cache_dir).expect("failed to create thumbnail cache dir");
            app.manage(cache::ThumbnailCache::new(cache_dir));
            let img_cache_dir = data_dir.join("img_cache");
            std::fs::create_dir_all(&img_cache_dir).expect("failed to create img cache dir");
            app.manage(cache::ImageCache::new(img_cache_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            library::scan_library,
            library::list_manga,
            library::search_manga,
            thumbnail::get_thumbnail,
            reader::list_pages,
            reader::get_page,
            metadata::set_score,
            metadata::get_manga_detail,
            metadata::update_manga_tags,
            eh_api::fetch_tags_by_url,
            eh_api::batch_fetch_tags,
            eh_api::retry_failed_tags,
            eh_api::set_eh_cookies,
            eh_api::search_eh_gallery,
            config_cmd::get_config,
            config_cmd::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}