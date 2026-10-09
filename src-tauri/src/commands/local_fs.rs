use std::fs;
use tauri::command;

#[derive(serde::Serialize)]
pub struct FileStat {
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct LocalFileEntry {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

#[command]
pub fn local_fs_read_dir(path: String) -> Result<Vec<LocalFileEntry>, String> {
    let mut entries = Vec::new();
    let dir = fs::read_dir(&path).map_err(|e| e.to_string())?;
    for entry in dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().ok();
        
        let (is_file, is_dir, size, modified) = if let Some(meta) = metadata {
            let mod_time = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs());
            (meta.is_file(), meta.is_dir(), meta.len(), mod_time)
        } else {
            (false, false, 0, None)
        };

        entries.push(LocalFileEntry {
            name,
            is_file,
            is_dir,
            size,
            modified,
        });
    }
    Ok(entries)
}

#[command]
pub fn local_fs_read_file(path: String) -> Result<Vec<u8>, String> {
    fs::read(&path).map_err(|e| e.to_string())
}

#[command]
pub fn local_fs_read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub fn local_fs_write_file(path: String, contents: Vec<u8>) -> Result<(), String> {
    fs::write(&path, contents).map_err(|e| e.to_string())
}

#[command]
pub fn local_fs_write_text_file(path: String, contents: String) -> Result<(), String> {
    fs::write(&path, contents).map_err(|e| e.to_string())
}

#[command]
pub fn local_fs_stat(path: String) -> Result<FileStat, String> {
    let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Ok(FileStat {
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified,
    })
}
#[command]
pub fn local_fs_rename(old_path: String, new_path: String) -> Result<(), String> {
    fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[command]
pub fn local_fs_mkdir(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}
#[command]
pub fn local_fs_remove(path: String) -> Result<(), String> {
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    if metadata.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&path).map_err(|e| e.to_string())
    }
}
