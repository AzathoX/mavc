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

    // Playlist exports handle an entire archive. Combining is built in; when
    // requested with the system player, explain the fallback and mix in Rodio.
    if args.browser_player {
        return open_browser_playlist(&args.archive, &archive.describe.title, &tracks);
    }
    if !args.combine.is_empty() {
        if args.system_player {
            eprintln!(
                "warning: --combine cannot be opened as a system playlist; using the built-in player"
            );
        }
        return play_combined(&args.archive, &archive.describe.title, &args.combine);
    }
    if args.system_player {
        return import_system_playlist(&args.archive, &tracks);
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

fn play_combined(
    archive_path: &Path,
    room_title: &str,
    track_ids: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    let output = DeviceSinkBuilder::open_default_sink()?;
    let mut players = Vec::with_capacity(track_ids.len());
    for &track_id in track_ids {
        let selected = play().archive(archive_path).one(track_id).stream()?;
        print_current_track(room_title, &selected.track);
        let source = Decoder::builder()
            .with_data(selected.bytes)
            .with_byte_len(selected.track.length)
            .with_hint(&selected.track.codec)
            .with_gapless(true)
            .build()?;
        let player = Player::connect_new(&output.mixer());
        player.append(source);
        players.push(player);
    }
    for player in players {
        player.sleep_until_end();
    }
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
    room_title: &str,
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
        entries.push_str(&track_info_html(room_title, track, &file_name));
    }
    let html = format!(
        "<!doctype html><html lang=\"zh-Hant\"><meta charset=\"utf-8\">\
         <title>音樂館 Music Room</title><body>\
         <h1>🎵 音樂館 Music Room：{}</h1>\
         <h2>歌曲信息 / Track info</h2><main>{entries}</main>\
         </body></html>",
        escape_html(room_title)
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

fn track_info_html(room_title: &str, track: &Track, file_name: &str) -> String {
    let title = track.metadata.title.as_deref().unwrap_or(&track.title);
    let mut details = format!("<dt>歌名 / Title</dt><dd>{}</dd>", escape_html(title));
    for (label, value) in [
        ("歌手 / Artist", track.metadata.artist.as_deref()),
        ("專輯 / Album", track.metadata.album.as_deref()),
        ("曲風 / Genre", track.metadata.genre.as_deref()),
        ("年份 / Year", track.metadata.year.as_deref()),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            details.push_str(&format!("<dt>{label}</dt><dd>{}</dd>", escape_html(value)));
        }
    }
    details.push_str(&format!(
        "<dt>曲目 ID</dt><dd>{}</dd><dt>格式 / Format</dt><dd>{}</dd>",
        track.id,
        escape_html(&track.codec)
    ));
    if let Some(duration) = track.metadata.duration_ms {
        details.push_str(&format!(
            "<dt>時長 / Duration</dt><dd>{}</dd>",
            format_duration(duration)
        ));
    }

    format!(
        "<section><h3>🎵 音樂館 Music Room：{}</h3>\
         <h4>歌曲信息 / Track info</h4><h5>{}</h5>\
         <dl>{details}</dl><audio controls preload=\"none\" src=\"{}\"></audio></section>",
        escape_html(room_title),
        escape_html(title),
        escape_html(file_name)
    )
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

#[cfg(target_os = "windows")]
fn open_with_system_player(path: &Path) -> io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_with_system_player(path: &Path) -> io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_with_system_player(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening a system playlist is unsupported on this platform",
    ))
}

#[cfg(target_os = "macos")]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_default_browser(path: &Path) -> io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_in_default_browser(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening a browser is unsupported on this platform",
    ))
}
