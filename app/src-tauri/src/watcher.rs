use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

/// Holds the active watcher. Replacing it drops (and stops) the previous one.
#[derive(Default)]
pub struct WatcherState(pub Mutex<Option<RecommendedWatcher>>);

/// Watch a single directory (non-recursive) and emit "fs-change" on any event.
/// Called whenever the active pane navigates; only the latest path is watched.
#[tauri::command]
pub fn watch_dir(
    app: AppHandle,
    state: State<WatcherState>,
    path: String,
) -> Result<(), String> {
    let handle = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if res.is_ok() {
            let _ = handle.emit("fs-change", ());
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    *state.0.lock().unwrap() = Some(watcher);
    Ok(())
}
