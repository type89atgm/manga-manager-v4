use super::ImageEntry;
use std::path::Path;

/// List images in a RAR archive.
/// - open_for_listing() returns Result<OpenArchive, UnrarError> → use `?`
/// - The OpenArchive is an iterator yielding Result<FileHeader, UnrarError>
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

/// Read a single image from RAR by entry name.
/// Extracts entire archive to temp dir (cached by file hash), then reads file.
pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let hash = {
        use sha2::{Digest, Sha256};
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        h.update(&bytes);
        format!("{:x}", h.finalize())[..16].to_string()
    };
    let extract_dir = std::env::temp_dir()
        .join("manga-manager-rar")
        .join(&hash);

    if !extract_dir.exists() {
        std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
        // extract_to takes ownership, so build path_buf first
        let archive = unrar::Archive::new(path.to_path_buf());
        archive.extract_to(&extract_dir).map_err(|e| e.to_string())?;
    }

    let file_path = extract_dir.join(entry_name);
    std::fs::read(&file_path).map_err(|e| e.to_string())
}
