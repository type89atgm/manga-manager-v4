use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub library_paths: Vec<String>,
    pub eh_cookies: EhCookies,
    pub thumbnail_size: u32,
    pub use_exhentai: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EhCookies {
    pub ipb_member_id: String,
    pub ipb_pass_hash: String,
    pub igneous: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            library_paths: Vec::new(),
            eh_cookies: EhCookies {
                ipb_member_id: String::new(),
                ipb_pass_hash: String::new(),
                igneous: String::new(),
            },
            thumbnail_size: 300,
            use_exhentai: true,
        }
    }
}

impl AppConfig {
    pub fn load(data_dir: &PathBuf) -> Self {
        let config_path = data_dir.join("config.json");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save(data_dir);
            config
        }
    }

    pub fn save(&self, data_dir: &PathBuf) {
        let config_path = data_dir.join("config.json");
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&config_path, content);
        }
    }
}