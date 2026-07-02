use crate::model::DriveInfo;
use sysinfo::Disks;

#[tauri::command]
pub fn list_drives() -> Vec<DriveInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut drives: Vec<DriveInfo> = disks
        .iter()
        .map(|d| DriveInfo {
            name: d.name().to_string_lossy().to_string(),
            path: d.mount_point().to_string_lossy().to_string(),
            total: d.total_space(),
            available: d.available_space(),
        })
        .collect();
    drives.sort_by(|a, b| a.path.cmp(&b.path));
    drives.dedup_by(|a, b| a.path == b.path);
    drives
}
