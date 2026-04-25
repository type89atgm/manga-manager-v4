use super::ImageEntry;
use std::path::Path;

/// List images in a RAR archive.
/// In unrar 0.5, open_for_listing() returns an OpenArchive (not Result),
/// and iterates over Result<FileHeader, UnrarError>.
pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let archive = unrar::Archive::new(path);
    let mut entries = Vec::new();
    for (i, entry) in archive.open_for_listing().enumerate() {
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
/// Extracts the whole archive to a temp dir (cached by file hash),
/// then reads the requested file from disk.
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
        let mut archive = unrar::Archive::new(path)
            .open_for_processing()
            .map_err(|e| e.to_string())?;
        loop {
            match archive.read_header().map_err(|e| e.to_string())? {
                Some(header) => {
                    if header.is_directory() {
                        archive = header.skip().map_err(|e| e.to_string())?;
                    } else {
                        archive = header.extract_with_base(&extract_dir).map_err(|e| e.to_string())?;
                    }
                }
                None => break,
            }
        }
    }

    let file_path = extract_dir.join(entry_name);
    std::fs::read(&file_path).map_err(|e| e.to_string())
}
