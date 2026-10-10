//! Keep OS file-open requests until the webview is ready to receive them.
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default)]
pub struct OpenFiles(Mutex<Vec<String>>);

fn archive_path(value: &str, cwd: &Path) -> Option<PathBuf> {
    let path = Path::new(value);
    if !path.extension()?.to_str()?.eq_ignore_ascii_case("mavc") {
        return None;
    }
    Some(if path.is_absolute() {
        path.to_owned()
    } else {
        cwd.join(path)
    })
}

impl OpenFiles {
    fn push(&self, paths: impl IntoIterator<Item = PathBuf>) -> bool {
        let mut pending = self.0.lock().unwrap_or_else(|error| error.into_inner());
        for path in paths {
            if let Some(path) = archive_path(&path.to_string_lossy(), Path::new(".")) {
                let path = path.to_string_lossy().into_owned();
                if !pending.contains(&path) {
                    pending.push(path);
                }
            }
        }
        !pending.is_empty()
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.0.lock().unwrap_or_else(|error| error.into_inner()))
    }
}

pub fn enqueue(app: &AppHandle, paths: impl IntoIterator<Item = PathBuf>) {
    let has_files = app.state::<OpenFiles>().push(paths);
    if has_files {
        // Notification is only a wake-up. The queue survives missed events.
        let _ = app.emit("malbum-open-files-changed", ());
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn from_args(app: &AppHandle, args: Vec<String>, cwd: &Path) {
    enqueue(
        app,
        args.iter().skip(1).filter_map(|arg| archive_path(arg, cwd)),
    );
}

#[tauri::command]
pub fn pending_open_files(state: tauri::State<'_, OpenFiles>) -> Vec<String> {
    state.take()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_archive_paths_and_preserves_spaces_and_unicode() {
        let cwd = std::env::temp_dir();
        assert_eq!(
            archive_path("曲目 collection.MAVC", &cwd),
            Some(cwd.join("曲目 collection.MAVC"))
        );
        let absolute = cwd.join("album.mavc");
        assert_eq!(
            archive_path(absolute.to_str().unwrap(), Path::new("unused")),
            Some(absolute)
        );
        for value in ["--help", "song.mp3", "album.mavc.exe", ""] {
            assert!(archive_path(value, &cwd).is_none());
        }
    }

    #[test]
    fn requests_survive_startup_and_can_be_received_again_after_drain() {
        let files = OpenFiles::default();
        let first = std::env::temp_dir().join("first.mavc");
        let second = std::env::temp_dir().join("second.MAVC");
        assert!(files.push([
            first.clone(),
            first.clone(),
            std::env::temp_dir().join("song.mp3")
        ]));
        assert_eq!(files.take(), vec![first.to_string_lossy().into_owned()]);
        assert!(files.take().is_empty());
        assert!(files.push([second.clone()]));
        assert_eq!(files.take(), vec![second.to_string_lossy().into_owned()]);
    }

    #[test]
    fn file_urls_decode_escaped_paths() {
        let path = std::env::temp_dir().join("音樂 album.mavc");
        let url = tauri::Url::from_file_path(&path).unwrap();
        assert_eq!(url.to_file_path().unwrap(), path);
    }
}
