use super::args::PlayArgs;
use mavc::{Archive, Track, play};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn play_random(args: PlayArgs) -> Result<(), Box<dyn std::error::Error>> {
    let archive = Archive::open(&args.archive)?;
    let tracks = archive.index.tracks.clone();

    // `--all` and the playlist destinations operate on the full archive. The
    // ordinary play command keeps its weighted-random / single-track behavior.
    if args.system_player {
        return import_system_playlist(&args.archive, &tracks);
    }
    if args.browser_player {
        return open_browser_playlist(&args.archive, &tracks);
    }
    if args.all {
        return play_all_in_order(&args.archive, &archive.describe.title, &tracks);
    }

    let selection = if let Some(id) = args.one {
        play().archive(&args.archive).one(id)
    } else {
        play().archive(&args.archive).random()
    };
    let selected = selection.stream()?;
    print_current_track(&archive.describe.title, &selected.track);
    let codec = selected.track.codec;
    let length = selected.track.length;
    let source = Decoder::builder()
        .with_data(selected.bytes)
        .with_byte_len(length)
        .with_hint(&codec)
        .with_gapless(true)
        .build()?;
    let output = DeviceSinkBuilder::open_default_sink()?;
    let player = Player::connect_new(&output.mixer());
    player.append(source);
    player.sleep_until_end();
    Ok(())
}

fn play_all_in_order(
    archive_path: &Path,
    room_title: &str,
    tracks: &[Track],
) -> Result<(), Box<dyn std::error::Error>> {
    if tracks.is_empty() {
        return Err("archive has no tracks".into());
    }
    let output = DeviceSinkBuilder::open_default_sink()?;
    for track in tracks {
        let selected = play().archive(archive_path).one(track.id).stream()?;
        print_current_track(room_title, &selected.track);
        let source = Decoder::builder()
            .with_data(selected.bytes)
            .with_byte_len(selected.track.length)
            .with_hint(&selected.track.codec)
            .with_gapless(true)
            .build()?;
        let player = Player::connect_new(&output.mixer());
        player.append(source);
        player.sleep_until_end();
    }
    Ok(())
}

fn print_current_track(room_title: &str, track: &Track) {
    let song_title = track.metadata.title.as_deref().unwrap_or(&track.title);
    println!("\n🎵 音樂館 Music Room：{}", room_title);
    println!("歌曲信息 / Track info");
    println!("  歌名 / Title：{song_title}");
    print_optional("歌手 / Artist", track.metadata.artist.as_deref());
    print_optional("專輯 / Album", track.metadata.album.as_deref());
    print_optional("曲風 / Genre", track.metadata.genre.as_deref());
    print_optional("年份 / Year", track.metadata.year.as_deref());
    println!("  曲目 ID：{}", track.id);
    println!("  格式 / Format：{}", track.codec);
    if let Some(duration) = track.metadata.duration_ms {
        println!("  時長 / Duration：{}", format_duration(duration));
    }
}

fn print_optional(label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        println!("  {label}：{value}");
    }
}

fn format_duration(duration_ms: u64) -> String {
    let seconds = duration_ms / 1_000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn import_system_playlist(
    archive_path: &Path,
    tracks: &[Track],
) -> Result<(), Box<dyn std::error::Error>> {
    if tracks.is_empty() {
        return Err("archive has no tracks".into());
    }
    let directory = create_temp_dir("mavc-playlist")?;
    let mut playlist = String::from("#EXTM3U\n");
    for track in tracks {
        let file_name = track_file_name(track);
        let mut selected = play().archive(archive_path).one(track.id).stream()?;
        io::copy(
            &mut selected.bytes,
            &mut File::create(directory.join(&file_name))?,
        )?;
        playlist.push_str(&format!("#EXTINF:-1,{}\n{}\n", track.title, file_name));
    }
    let playlist_path = directory.join("playlist.m3u8");
    File::create(&playlist_path)?.write_all(playlist.as_bytes())?;
    open_with_system_player(&playlist_path)?;
    println!(
        "Opened playlist with {} tracks: {}",
        tracks.len(),
        playlist_path.display()
    );
    Ok(())
}

fn open_browser_playlist(
    archive_path: &Path,
    tracks: &[Track],
) -> Result<(), Box<dyn std::error::Error>> {
    if tracks.is_empty() {
        return Err("archive has no tracks".into());
    }
    let directory = create_temp_dir("mavc-browser")?;
    let mut entries = String::new();
    for track in tracks {
        let file_name = track_file_name(track);
        let mut selected = play().archive(archive_path).one(track.id).stream()?;
        io::copy(
            &mut selected.bytes,
            &mut File::create(directory.join(&file_name))?,
        )?;
        entries.push_str(&format!(
            "<li><strong>{}</strong> <audio controls preload=\"none\" src=\"{}\"></audio></li>",
            escape_html(&track.title),
            file_name
        ));
    }
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>MAVC playlist</title>\
         <h1>MAVC playlist</h1><ol>{entries}</ol>"
    );
    let html_path = directory.join("player.html");
    File::create(&html_path)?.write_all(html.as_bytes())?;
    open_in_default_browser(&html_path)?;
    println!(
        "Opened track selection page with {} tracks: {}",
        tracks.len(),
        html_path.display()
    );
    Ok(())
}

fn track_file_name(track: &Track) -> String {
    format!("track-{}.{}", track.id, track.codec)
}

fn create_temp_dir(prefix: &str) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("{prefix}-{}-{timestamp}", std::process::id()));
    std::fs::create_dir(&directory)?;
    Ok(directory)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(target_os = "macos")]
fn open_with_system_player(path: &Path) -> io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_with_system_player(path: &Path) -> io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_with_system_player(path: &Path) -> io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_with_system_player(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening audio with the system player is unsupported on this platform",
    ))
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_in_default_browser(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening a browser is unsupported on this platform",
    ))
}
