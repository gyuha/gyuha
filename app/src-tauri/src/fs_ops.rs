use crate::model::{DirListing, FileEntry};
use fs_extra::TransitProcess;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter};

/// Core directory reader — pure (no Tauri), so it is unit-testable.
pub fn read_dir(path: &Path) -> Result<DirListing, String> {
    let meta = fs::metadata(path).map_err(|e| format!("경로에 접근할 수 없습니다: {e}"))?;
    if !meta.is_dir() {
        return Err("디렉토리가 아닙니다".into());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let p = entry.path();
        let is_symlink = entry
            .file_type()
            .map(|t| t.is_symlink())
            .unwrap_or(false);

        // Follow symlinks for size/is_dir; fall back to the link metadata.
        let md = fs::metadata(&p).or_else(|_| fs::symlink_metadata(&p));
        let (is_dir, size, modified) = match md {
            Ok(m) => {
                let modified = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64);
                let is_dir = m.is_dir();
                (is_dir, if is_dir { 0 } else { m.len() }, modified)
            }
            Err(_) => (false, 0, None),
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let extension = if is_dir {
            String::new()
        } else {
            p.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        entries.push(FileEntry {
            name,
            path: p.to_string_lossy().to_string(),
            is_dir,
            is_symlink,
            size,
            modified,
            extension,
        });
    }

    let parent = path.parent().map(|p| p.to_string_lossy().to_string());
    Ok(DirListing {
        path: path.to_string_lossy().to_string(),
        parent,
        entries,
    })
}

#[tauri::command]
pub fn list_dir(path: String) -> Result<DirListing, String> {
    read_dir(Path::new(&path))
}

#[tauri::command]
pub fn home_dir() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".into())
}

#[tauri::command]
pub fn create_dir(parent: String, name: String) -> Result<String, String> {
    let p = Path::new(&parent).join(&name);
    fs::create_dir(&p).map_err(|e| e.to_string())?;
    Ok(p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn rename_path(path: String, new_name: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let parent = p.parent().ok_or("상위 경로를 찾을 수 없습니다")?;
    let dest = parent.join(&new_name);
    if dest.exists() {
        return Err("같은 이름의 항목이 이미 존재합니다".into());
    }
    fs::rename(&p, &dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn delete_paths(paths: Vec<String>, to_trash: bool) -> Result<(), String> {
    for path in &paths {
        if to_trash {
            trash::delete(path).map_err(|e| e.to_string())?;
        } else {
            let p = Path::new(path);
            let md = fs::symlink_metadata(p).map_err(|e| e.to_string())?;
            if md.is_dir() {
                fs::remove_dir_all(p).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(p).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| e.to_string())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    current_file: String,
    copied: u64,
    total: u64,
}

fn copy_options() -> fs_extra::dir::CopyOptions {
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.copy_inside = true;
    options
}

#[tauri::command]
pub fn copy_paths(app: AppHandle, paths: Vec<String>, dest_dir: String) -> Result<(), String> {
    let handle = app.clone();
    let progress = move |info: TransitProcess| {
        let _ = handle.emit(
            "copy-progress",
            Progress {
                current_file: info.file_name.clone(),
                copied: info.copied_bytes,
                total: info.total_bytes,
            },
        );
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    };
    fs_extra::copy_items_with_progress(&paths, &dest_dir, &copy_options(), progress)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("copy-done", &dest_dir);
    Ok(())
}

#[tauri::command]
pub fn move_paths(app: AppHandle, paths: Vec<String>, dest_dir: String) -> Result<(), String> {
    let handle = app.clone();
    let progress = move |info: TransitProcess| {
        let _ = handle.emit(
            "copy-progress",
            Progress {
                current_file: info.file_name.clone(),
                copied: info.copied_bytes,
                total: info.total_bytes,
            },
        );
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    };
    fs_extra::move_items_with_progress(&paths, &dest_dir, &copy_options(), progress)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("copy-done", &dest_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("gc_test_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn read_dir_lists_files_and_dirs() {
        let dir = scratch("read");
        fs::write(dir.join("a.txt"), b"hello").unwrap();
        fs::create_dir(dir.join("sub")).unwrap();

        let listing = read_dir(&dir).unwrap();
        assert!(listing.parent.is_some());
        let file = listing.entries.iter().find(|e| e.name == "a.txt").unwrap();
        assert!(!file.is_dir);
        assert_eq!(file.size, 5);
        assert_eq!(file.extension, "txt");
        let sub = listing.entries.iter().find(|e| e.name == "sub").unwrap();
        assert!(sub.is_dir);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn read_dir_rejects_files() {
        let dir = scratch("rejectfile");
        let f = dir.join("x.txt");
        fs::write(&f, b"x").unwrap();
        assert!(read_dir(&f).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn create_and_rename_and_delete() {
        let dir = scratch("ops");
        let created = create_dir(dir.to_string_lossy().to_string(), "newdir".into()).unwrap();
        assert!(Path::new(&created).is_dir());

        let renamed = rename_path(created.clone(), "renamed".into()).unwrap();
        assert!(!Path::new(&created).exists());
        assert!(Path::new(&renamed).is_dir());

        delete_paths(vec![renamed.clone()], false).unwrap();
        assert!(!Path::new(&renamed).exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rename_refuses_overwrite() {
        let dir = scratch("norename");
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();
        let res = rename_path(a.to_string_lossy().to_string(), "b.txt".into());
        assert!(res.is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
