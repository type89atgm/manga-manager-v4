pub mod rar_reader;
pub mod zip_reader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub name: String,
    pub index: usize,
}

fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png")
        || lower.ends_with(".gif") || lower.ends_with(".bmp") || lower.ends_with(".webp")
}

pub fn list_images(path: &Path) -> Result<Vec<ImageEntry>, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mut entries = match ext.as_str() {
        "zip" | "cbz" => zip_reader::list_images(path)?,
        "rar" | "cbr" => rar_reader::list_images(path)?,
        _ => return Err(format!("unsupported: {}", ext)),
    };
    entries.retain(|e| is_image_file(&e.name));
    entries.sort_by(|a, b| natord::compare(&a.name, &b.name));
    for (i, e) in entries.iter_mut().enumerate() {
        e.index = i;
    }
    Ok(entries)
}

pub fn read_image(path: &Path, entry_name: &str) -> Result<Vec<u8>, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "zip" | "cbz" => zip_reader::read_image(path, entry_name),
        "rar" | "cbr" => rar_reader::read_image(path, entry_name),
        _ => Err(format!("unsupported: {}", ext)),
    }
}