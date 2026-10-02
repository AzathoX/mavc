//! Tauri command handlers that adapt the UI to the MAVC library and desktop APIs.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use lofty::{
    file::{FileType, TaggedFileExt},
    probe::Probe,
};
use mavc::{
    add_tracks, create_archive, music, play, remove_tracks, Archive, CreateManifest, Track,
};
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfo {
    pub path: String,
    pub title: String,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub tracks: Vec<TrackInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    #[serde(flatten)]
    pub track: Track,
    pub embedded_cover_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackData {
    pub archive_title: String,
    pub track: Track,
    pub src: String,
}

// Shared archive loading and path conversion helpers.
fn paths(values: Vec<String>) -> Vec<PathBuf> {
    values.into_iter().map(PathBuf::from).collect()
}

fn load_archive(path: &str) -> Result<(Archive, ArchiveInfo), String> {
    let archive = Archive::open(path).map_err(|error| error.to_string())?;
    let mut tracks = Vec::with_capacity(archive.index.tracks.len());
    for mut track in archive.index.tracks.clone() {
        track.metadata = archive
            .track_metadata(path, track.id)
            .map_err(|error| error.to_string())?;
        let embedded_cover_data_url = match track.codec.as_str() {
            "flac" => archive
                .open_track(path, track.id)
                .ok()
                .and_then(|reader| Probe::with_file_type(reader, FileType::Flac).read().ok())
                .and_then(|file| {
                    file.tags()
                        .iter()
                        .flat_map(|tag| tag.pictures())
                        .next()
                        .map(|picture| {
                            let mime = picture
                                .mime_type()
                                .map(|mime| mime.as_str())
                                .unwrap_or("image/jpeg");
                            format!("data:{mime};base64,{}", STANDARD.encode(picture.data()))
                        })
                }),
            _ => None,
        };
        tracks.push(TrackInfo {
            track,
            embedded_cover_data_url,
        });
    }
    let info = ArchiveInfo {
        path: path.to_owned(),
        title: archive.describe.title.clone(),
        creator: archive.describe.creator.clone(),
        description: archive.describe.description.clone(),
        tracks,
    };
    Ok((archive, info))
}

// Archive inspection and creation commands.
#[tauri::command]
pub fn archive_list(path: String) -> Result<ArchiveInfo, String> {
    load_archive(&path).map(|(_, info)| info)
}

#[tauri::command]
pub fn archive_inspect(path: String, track_id: Option<usize>) -> Result<ArchiveInfo, String> {
    let (_, mut info) = load_archive(&path)?;
    if let Some(id) = track_id {
        info.tracks.retain(|track| track.track.id == id);
        if info.tracks.is_empty() {
            return Err(format!("no track with id {id}"));
        }
    }
    Ok(info)
}

#[tauri::command]
pub fn archive_create_music(files: Vec<String>, output: Option<String>) -> Result<String, String> {
    if files.is_empty() {
        return Err("請至少選擇一個音訊檔案".into());
    }
    let mut builder = music().from(paths(files));
    if let Some(output) = output.filter(|value| !value.trim().is_empty()) {
        builder = builder.output(output);
    }
    builder
        .create()
        .map(|path| format!("已建立 MAVC 封存：{}", path.display()))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn archive_create_music_from_picker(
    app: tauri::AppHandle,
) -> Result<Option<ArchiveInfo>, String> {
    let Some(selected_files) = app
        .dialog()
        .file()
        .add_filter(
            "Audio files",
            &["mp3", "flac", "wav", "m4a", "aac", "ogg", "opus"],
        )
        .blocking_pick_files()
    else {
        return Ok(None);
    };

    if selected_files.len() < 2 {
        return Err("Please select at least two audio files.".into());
    }

    let files = selected_files
        .into_iter()
        .map(|file| {
            file.into_path()
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let output_path = music()
        .from(paths(files))
        .create()
        .map_err(|error| error.to_string())?;
    load_archive(&output_path.to_string_lossy()).map(|(_, info)| Some(info))
}

#[tauri::command]
pub async fn archive_pick_existing(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let Some(selected_file) = app
        .dialog()
        .file()
        .add_filter("MAVC archive", &["mavc"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    selected_file
        .into_path()
        .map(|path| Some(path.to_string_lossy().into_owned()))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn archive_pick_audio(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let Some(file) = app
        .dialog()
        .file()
        .add_filter(
            "Audio files",
            &["mp3", "flac", "wav", "m4a", "aac", "ogg", "opus"],
        )
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    file.into_path()
        .map(|path| Some(path.to_string_lossy().into_owned()))
        .map_err(|e| e.to_string())
}

// File picker and track metadata commands.
#[tauri::command]
pub fn archive_update_track_details(
    path: String,
    track_id: usize,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    cover_art_url: Option<String>,
    weight: f64,
) -> Result<(), String> {
    Archive::update_track_details(
        path,
        track_id,
        title,
        artist,
        album,
        genre,
        year,
        cover_art_url,
        weight,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn archive_export_picker(
    app: tauri::AppHandle,
    source: String,
) -> Result<Option<String>, String> {
    let source_path = PathBuf::from(&source);
    let stem = source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("archive");
    let Some(destination) = app
        .dialog()
        .file()
        .set_file_name(format!("{stem}.mavc"))
        .add_filter("MAVC archive (*.mavc)", &["mavc"])
        .add_filter("MVSC archive (*.mvsc)", &["mvsc"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let destination = destination.into_path().map_err(|error| error.to_string())?;
    let extension = destination
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "mavc" | "mvsc") {
        return Err("Please choose the .mavc or .mvsc file type.".into());
    }
    std::fs::copy(&source_path, &destination).map_err(|error| error.to_string())?;
    Ok(Some(destination.to_string_lossy().into_owned()))
}

// Archive mutation and extraction commands.
#[tauri::command]
pub fn archive_create_manifest(manifest_path: String, output: String) -> Result<String, String> {
    let manifest_path = PathBuf::from(manifest_path);
    let output = PathBuf::from(output);
    let bytes = std::fs::read(&manifest_path).map_err(|error| error.to_string())?;
    let mut manifest: CreateManifest =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    for track in &mut manifest.tracks {
        if track.path.is_relative() {
            track.path = base.join(&track.path);
        }
    }
    let count = manifest.tracks.len();
    create_archive(&manifest, &output).map_err(|error| error.to_string())?;
    Ok(format!("已建立封存 {}（{count} 首歌曲）", output.display()))
}

#[tauri::command]
pub fn archive_add(archives: Vec<String>, files: Vec<String>) -> Result<String, String> {
    let archive_paths = paths(archives);
    let audio_paths = paths(files);
    add_tracks(&archive_paths, &audio_paths).map_err(|error| error.to_string())?;
    Ok(format!(
        "已將 {} 個音訊檔加入 {} 個封存",
        audio_paths.len(),
        archive_paths.len()
    ))
}

#[tauri::command]
pub fn archive_remove(archives: Vec<String>, track_ids: Vec<usize>) -> Result<String, String> {
    let archive_paths = paths(archives);
    remove_tracks(&archive_paths, &track_ids).map_err(|error| error.to_string())?;
    Ok(format!(
        "已從 {} 個封存移除 {} 首歌曲",
        archive_paths.len(),
        track_ids.len()
    ))
}

#[tauri::command]
pub fn archive_pick(path: String) -> Result<Track, String> {
    let archive = Archive::open(&path).map_err(|error| error.to_string())?;
    archive
        .pick_random()
        .cloned()
        .ok_or_else(|| "沒有權重大於零的歌曲可供抽選".into())
}

#[tauri::command]
pub fn archive_weight(path: String, track_id: usize, weight: f64) -> Result<String, String> {
    let (old, new) =
        Archive::update_track_weight(path, track_id, weight).map_err(|error| error.to_string())?;
    Ok(format!("歌曲 {track_id} 權重已由 {old} 更新為 {new}"))
}

#[tauri::command]
pub fn archive_extract_one(
    path: String,
    track_id: usize,
    output: String,
) -> Result<String, String> {
    let archive = Archive::open(&path).map_err(|error| error.to_string())?;
    archive
        .extract_track(&path, track_id, &output)
        .map_err(|error| error.to_string())?;
    Ok(format!("已抽取歌曲 {track_id} 至 {output}"))
}

#[tauri::command]
pub fn archive_extract_all(path: String, output_dir: String) -> Result<String, String> {
    let archive = Archive::open(&path).map_err(|error| error.to_string())?;
    if archive.index.tracks.is_empty() {
        return Err("封存中沒有歌曲可抽取".into());
    }
    let output_dir = PathBuf::from(output_dir);
    let mut names = std::collections::HashSet::new();
    for track in &archive.index.tracks {
        let file_name = Path::new(&track.file_name)
            .file_name()
            .ok_or_else(|| "歌曲沒有可用的檔名".to_owned())?;
        let collision_key = file_name.to_string_lossy().to_lowercase();
        if !names.insert(collision_key) {
            return Err(format!(
                "歌曲檔名 {} 重複，為避免覆寫已取消",
                file_name.to_string_lossy()
            ));
        }
        let destination = output_dir.join(file_name);
        if destination.exists() {
            return Err(format!(
                "{} 已存在，為避免覆寫已取消",
                destination.display()
            ));
        }
    }
    std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    for track in &archive.index.tracks {
        let file_name = Path::new(&track.file_name)
            .file_name()
            .ok_or_else(|| "歌曲沒有可用的檔名".to_owned())?;
        archive
            .extract_track(&path, track.id, output_dir.join(file_name))
            .map_err(|error| error.to_string())?;
    }
    Ok(format!(
        "已抽取 {} 首歌曲至 {}",
        archive.index.tracks.len(),
        output_dir.display()
    ))
}

// Playback URL creation and system-player integration.
#[tauri::command]
pub async fn archive_play_track(
    app: tauri::AppHandle,
    path: String,
    track_id: Option<usize>,
) -> Result<PlaybackData, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut selected = if let Some(id) = track_id {
            play().archive(&path).one(id).stream()
        } else {
            play().archive(&path).random().stream()
        }
        .map_err(|error| error.to_string())?;
        let archive_title = Archive::open(&path)
            .map_err(|error| error.to_string())?
            .describe
            .title;
        let src = playback_asset_url(&app, &selected.track, &mut selected.bytes)?;
        Ok(PlaybackData {
            archive_title,
            track: selected.track,
            src,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn archive_play_combine(
    app: tauri::AppHandle,
    path: String,
    track_ids: Vec<usize>,
) -> Result<Vec<PlaybackData>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let archive_title = Archive::open(&path)
            .map_err(|error| error.to_string())?
            .describe
            .title;
        play()
            .archive(&path)
            .combine(track_ids)
            .streams()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|mut selected| {
                let src = playback_asset_url(&app, &selected.track, &mut selected.bytes)?;
                Ok(PlaybackData {
                    archive_title: archive_title.clone(),
                    track: selected.track,
                    src,
                })
            })
            .collect()
    })
    .await
    .map_err(|error| error.to_string())?
}

fn playback_asset_url(
    app: &tauri::AppHandle,
    track: &Track,
    audio: &mut impl Read,
) -> Result<String, String> {
    let directory = std::env::temp_dir().join("mavc-playback");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "{}-{}-{}.{}",
        std::process::id(),
        track.id,
        timestamp(),
        track.codec
    ));
    let mut output = File::create(&path).map_err(|error| error.to_string())?;
    std::io::copy(audio, &mut output).map_err(|error| error.to_string())?;
    drop(output);
    app.asset_protocol_scope()
        .allow_file(&path)
        .map_err(|error| error.to_string())?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主視窗不存在，無法載入歌曲。".to_owned())?;
    window
        .convert_file_src(&path, None)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn archive_system_playlist(path: String) -> Result<String, String> {
    let archive = Archive::open(&path).map_err(|error| error.to_string())?;
    if archive.index.tracks.is_empty() {
        return Err("封存中沒有歌曲可加入播放清單".into());
    }
    let directory = std::env::temp_dir().join(format!(
        "mavc-playlist-{}-{}",
        std::process::id(),
        timestamp()
    ));
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let mut playlist = String::from("#EXTM3U\n");
    for track in &archive.index.tracks {
        let file_name = format!("track-{}.{}", track.id, track.codec);
        archive
            .extract_track(&path, track.id, directory.join(&file_name))
            .map_err(|error| error.to_string())?;
        playlist.push_str(&format!("#EXTINF:-1,{}\n{}\n", track.title, file_name));
    }
    let playlist_path = directory.join("playlist.m3u8");
    File::create(&playlist_path)
        .and_then(|mut file| file.write_all(playlist.as_bytes()))
        .map_err(|error| error.to_string())?;
    open_with_system_player(&playlist_path).map_err(|error| error.to_string())?;
    Ok(format!(
        "已交由系統播放器開啟 {} 首歌曲",
        archive.index.tracks.len()
    ))
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

#[cfg(target_os = "macos")]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_with_system_player(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "系統播放器不支援此平台",
    ))
}
