use super::ImageEntry;
use std::path::Path;

/// List images in RAR archive.
/// In unrar 0.5, open_for_listing() yields Entry directly (not Result).
pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let archive = unrar::Archive::new(path);
    let mut entries = Vec::new();
    for (i, entry) in archive.open_for_listing()
        .map_err(|e| e.to_string())?
        .enumerate()
    {
        if !entry.is_directory() {
            entries.push(ImageEntry {
                name: entry.filename.display().to_string(),
                index: i,
            });
        }
    }
    Ok(entries)
}

/// Read a single image from RAR by entry name.
/// open_for_processing is not an Iterator in unrar 0.5,
/// so we extract the archive to a temp dir and read the file.
pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(path.to_string_lossy().as_bytes());
        format!("{:x}", h.finalize())[..16].to_string()
    };
    let extract_dir = std::env::temp_dir()
        .join("manga-manager-rar")
        .join(&hash);

    if !extract_dir.exists() {
        std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
        unrar::Archive::new(path)
            .extract_to(&extract_dir)
            .map_err(|e| e.to_string())?;
    }

    let file_path = extract_dir.join(entry_name);
    std::fs::read(&file_path).map_err(|e| e.to_string())
}
