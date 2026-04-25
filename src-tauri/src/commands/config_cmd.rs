use crate::config::AppConfig;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_config(config: State<'_, Mutex<AppConfig>>) -> Result<AppConfig, String> {
    let config = config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(
    library_paths: Option<Vec<String>>,
    thumbnail_size: Option<u32>,
    use_exhentai: Option<bool>,
    config: State<'_, Mutex<AppConfig>>,
) -> Result<(), String> {
    let mut config = config.lock().map_err(|e| e.to_string())?;
    if let Some(p) = library_paths { config.library_paths = p; }
    if let Some(s) = thumbnail_size { config.thumbnail_size = s; }
    if let Some(u) = use_exhentai { config.use_exhentai = u; }
    let data_dir = dirs::data_dir().ok_or("no data dir")?.join("manga-manager");
    config.save(&data_dir);
    Ok(())
}