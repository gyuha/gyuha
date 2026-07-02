mod drives;
mod fs_ops;
mod model;
mod watcher;

use watcher::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState::default())
        .invoke_handler(tauri::generate_handler![
            fs_ops::list_dir,
            fs_ops::home_dir,
            fs_ops::create_dir,
            fs_ops::rename_path,
            fs_ops::delete_paths,
            fs_ops::copy_paths,
            fs_ops::move_paths,
            fs_ops::open_path,
            drives::list_drives,
            watcher::watch_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
