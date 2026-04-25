use super::ImageEntry;
use std::path::Path;

/// List images in a RAR archive.
/// open_for_listing() returns Result<OpenArchive>, which IS an iterator.
pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let archive = unrar::Archive::new(path);
    let list = archive.open_for_listing().map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for (i, entry) in list.enumerate() {
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

/// Read a single image from a RAR archive into memory.
/// Uses open_for_processing + read_header loop; only the target file is extracted.
pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let archive = unrar::Archive::new(path);
    let mut proc = archive
        .open_for_processing()
        .map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir().join("manga-manager-rar");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    loop {
        match proc.read_header().map_err(|e| e.to_string())? {
            Some(header) => {
                let info = header.entry();
                let name = info.filename.display().to_string();
                if info.is_directory() {
                    proc = header.skip().map_err(|e| e.to_string())?;
                } else if name == entry_name {
                    proc = header
                        .extract_with_base(&temp_dir)
                        .map_err(|e| e.to_string())?;
                    let file_path = temp_dir.join(&name);
                    let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
                    let _ = std::fs::remove_file(&file_path);
                    return Ok(data);
                } else {
                    proc = header.skip().map_err(|e| e.to_string())?;
                }
            }
            None => break,
        }
    }
    Err("image not found in rar".into())
}
