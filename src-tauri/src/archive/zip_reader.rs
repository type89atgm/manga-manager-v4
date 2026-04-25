use super::ImageEntry;
use std::io::Read;
use std::path::Path;

pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let f = archive.by_index(i).map_err(|e| e.to_string())?;
        if !f.is_dir() {
            entries.push(ImageEntry {
                name: f.name().to_string(),
                index: i,
            });
        }
    }
    Ok(entries)
}

pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut zf = archive.by_name(entry_name).map_err(|e| e.to_string())?;
    let mut buf = Vec::with_capacity(zf.size() as usize);
    zf.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}