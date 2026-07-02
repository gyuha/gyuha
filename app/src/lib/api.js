import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const listDir = (path) => invoke("list_dir", { path });
export const homeDir = () => invoke("home_dir");
export const listDrives = () => invoke("list_drives");
export const createDir = (parent, name) => invoke("create_dir", { parent, name });
export const renamePath = (path, newName) => invoke("rename_path", { path, newName });
export const deletePaths = (paths, toTrash) => invoke("delete_paths", { paths, toTrash });
export const copyPaths = (paths, destDir) => invoke("copy_paths", { paths, destDir });
export const movePaths = (paths, destDir) => invoke("move_paths", { paths, destDir });
export const openPath = (path) => invoke("open_path", { path });
export const watchDir = (path) => invoke("watch_dir", { path });

export const onFsChange = (cb) => listen("fs-change", cb);
export const onCopyProgress = (cb) => listen("copy-progress", (e) => cb(e.payload));
export const onCopyDone = (cb) => listen("copy-done", cb);
