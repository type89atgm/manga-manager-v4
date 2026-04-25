use super::ImageEntry;
use std::path::Path;

/// Memory-based RAR reading - no temp files on disk.
/// Uses unrar 0.5 open_for_listing / open_for_processing API.

pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let archive = unrar::Archive::new(path);
    let mut entries = Vec::new();
    for (i, entry) in archive.open_for_listing().into_iter().enumerate() {
        let e = entry.map_err(|e| e.to_string())?;
        if !e.is_directory() {
            entries.push(ImageEntry {
                name: e.filename.display().to_string(),
                index: i,
            });
        }
    }
    Ok(entries)
}

pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let archive = unrar::Archive::new(path);
    let mut result = Vec::new();
    for entry in archive.open_for_processing().map_err(|e| e.to_string())? {
        let mut e = entry.map_err(|e| e.to_string())?;
        if e.filename.display().to_string() == entry_name {
            if e.is_file() {
                result = e.extract_to_vec().map_err(|e| e.to_string())?;
            }
            break;
        }
    }
    if result.is_empty() {
        Err("image not found in rar".into())
    } else {
        Ok(result)
    }
}