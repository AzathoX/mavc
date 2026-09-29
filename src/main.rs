use mavc::{Archive, AudioMetadata, CreateManifest, Description, TrackSource, create_archive};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

fn usage() {
    eprintln!(
        "mavc — weighted music archive tool\n\
         Usage:\n\
           mavc help|--help|-h\n\
           mavc version|--version|-V|-v\n\
           mavc package -m <audio-file>... [-a <output.mavc>]\n\
           mavc package <manifest.json> <output.mavc>\n\
           mavc music -m <audio-file>... [-a <output.mavc>]\n\
           mavc music -a <output.mavc> -m <audio-file>...\n\
           mavc create <manifest.json> <output.mavc>\n\
           mavc list <archive.mavc>\n\
           mavc play [-r|--random | -o|--one <track-id>] [-sp|--system-player | -bp|--browser-player] <archive.mavc>\n\
           mavc inspect <archive.mavc> [track-id]\n\
           mavc inspect <track-id> <archive.mavc>\n\
           mavc pick <archive.mavc>\n\
           mavc weight <archive.mavc> <track-id>=<weight>\n\
           mavc extract <archive.mavc> <track-id> [output-file]"
    );
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if matches!(
        args.get(1).map(String::as_str),
        Some("help" | "--help" | "-h")
    ) {
        usage();
        return Ok(());
    }
    if matches!(
        args.get(1).map(String::as_str),
        Some("--VERSION" | "-v" | "--version" | "-V")
    ) {
        println!("mavc {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("music") {
        return create_from_music_list(&args[2..]);
    }
    if args.get(1).map(String::as_str) == Some("package") {
        if args.get(2).is_some_and(|arg| arg.starts_with('-')) {
            return create_from_music_list(&args[2..]);
        }
        if args.len() == 4 {
            return create_from_manifest(&args[2], &args[3]);
        }
        usage();
        return Err("package expects a music list or a manifest and output path".into());
    }
    // Keep the shorter form available as well: mavc -m file.mp3
    if matches!(args.get(1).map(String::as_str), Some("-m" | "--music-list")) {
        return create_from_music_list(&args[2..]);
    }
    match args.get(1).map(String::as_str) {
        Some("play") => play_random(&args[2..])?,
        Some("list") if args.len() == 3 => {
            let archive = Archive::open(&args[2])?;
            println!(
                "{} — {} track(s)",
                archive.describe.title,
                archive.index.tracks.len()
            );
            for track in &archive.index.tracks {
                println!(
                    "{}\t{}\t{}\tweight={}",
                    track.id, track.title, track.codec, track.weight
                );
            }
        }
        Some("create") if args.len() == 4 => {
            create_from_manifest(&args[2], &args[3])?;
        }
        Some("inspect") if (args.len() == 3 || args.len() == 4) => {
            inspect_archive(&args[2..])?;
        }
        Some("pick") if args.len() == 3 => {
            let archive = Archive::open(&args[2])?;
            match archive.pick_random() {
                Some(track) => println!("{}\t{}\t{}", track.id, track.title, track.file_name),
                None => return Err("no track has a positive random-play weight".into()),
            }
        }
        Some("weight") if args.len() == 4 => {
            let (track_id, weight) = args[3]
                .split_once('=')
                .ok_or("weight assignment must use <track-id>=<weight>")?;
            let track_id: usize = track_id.parse()?;
            let weight: f64 = weight.parse()?;
            let (old_weight, new_weight) =
                Archive::update_track_weight(&args[2], track_id, weight)?;
            println!("The weight update succeeded: from {old_weight} to {new_weight}.");
        }
        Some("extract") if args.len() == 4 || args.len() == 5 => {
            let archive = Archive::open(&args[2])?;
            let id: usize = args[3].parse()?;
            let output = match args.get(4) {
                Some(path) => PathBuf::from(path),
                None => {
                    let track = archive
                        .index
                        .tracks
                        .iter()
                        .find(|track| track.id == id)
                        .ok_or_else(|| format!("no track with id {id}"))?;
                    PathBuf::from(
                        Path::new(&track.file_name)
                            .file_name()
                            .ok_or("track has no usable filename")?,
                    )
                }
            };
            archive.extract_track(&args[2], id, &output)?;
            println!("Extracted track {id} to {}", output.display());
        }
        _ => {
            usage();
            return Err("invalid command or arguments".into());
        }
    }
    Ok(())
}

fn inspect_archive(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let (archive_path, track_id) = if args.len() == 1 {
        (args[0].as_str(), None)
    } else if let Ok(id) = args[0].parse::<usize>() {
        (args[1].as_str(), Some(id))
    } else {
        (args[0].as_str(), Some(args[1].parse::<usize>()?))
    };
    let archive = Archive::open(archive_path)?;
    println!("Archive: {}", archive.describe.title);
    if let Some(creator) = &archive.describe.creator {
        println!("Creator: {creator}");
    }
    if let Some(description) = &archive.describe.description {
        println!("Description: {description}");
    }
    if let Some(id) = track_id {
        let track = archive
            .index
            .tracks
            .iter()
            .find(|track| track.id == id)
            .ok_or_else(|| format!("no track with id {id}"))?;
        let metadata = archive.track_metadata(archive_path, id)?;
        print_track_details(track, &metadata);
    } else {
        println!("Tracks ({}):", archive.index.tracks.len());
        for track in &archive.index.tracks {
            let metadata = archive.track_metadata(archive_path, track.id)?;
            print_track_details(track, &metadata);
        }
    }
    Ok(())
}

fn print_track_details(track: &mavc::Track, metadata: &AudioMetadata) {
    println!("\nTrack {}", track.id);
    println!("  Name: {}", track.title);
    println!(
        "  File: {} ({}, {} bytes)",
        track.file_name, track.codec, track.length
    );
    println!("  Weight: {}", track.weight);
    print_optional("Title", metadata.title.as_deref());
    print_optional("Singer/artist", metadata.artist.as_deref());
    print_optional("Album", metadata.album.as_deref());
    print_optional("Genre", metadata.genre.as_deref());
    print_optional("Year", metadata.year.as_deref());
    print_optional("Comment", metadata.comment.as_deref());
    if let Some(track_number) = metadata.track_number {
        println!("  Track number: {track_number}");
    }
    if let Some(duration_ms) = metadata.duration_ms {
        let seconds = duration_ms / 1000;
        println!("  Duration: {}:{:02}", seconds / 60, seconds % 60);
    }
    if let Some(bitrate) = metadata.bitrate_kbps {
        println!("  Bitrate: {bitrate} kbps");
    }
    if let Some(sample_rate) = metadata.sample_rate_hz {
        println!("  Sample rate: {sample_rate} Hz");
    }
    if let Some(channels) = metadata.channels {
        println!("  Channels: {channels}");
    }
    if let Some(bit_depth) = metadata.bit_depth {
        println!("  Bit depth: {bit_depth} bit");
    }
}

fn print_optional(label: &str, value: Option<&str>) {
    if let Some(value) = value {
        println!("  {label}: {value}");
    }
}

fn play_random(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive_path = None;
    let mut use_system_player = false;
    let mut use_browser_player = false;
    let mut selected_id = None;
    let mut random_selected = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-r" | "--random" => random_selected = true,
            "-o" | "--one" => {
                i += 1;
                let id = args
                    .get(i)
                    .ok_or("-o/--one requires a track id")?
                    .parse::<usize>()?;
                selected_id = Some(id);
            }
            "-sp" | "--system-player" => use_system_player = true,
            "-bp" | "--browser-player" | "--broswer-player" => use_browser_player = true,
            option if option.starts_with('-') => {
                return Err(format!("unknown play option '{option}'").into());
            }
            path if archive_path.is_none() => archive_path = Some(path),
            _ => return Err("play accepts one .mavc archive".into()),
        }
        i += 1;
    }
    if random_selected && selected_id.is_some() {
        return Err("choose either -r/--random or -o/--one".into());
    }
    if use_system_player && use_browser_player {
        return Err("choose either --system-player or --browser-player".into());
    }
    let archive_path = archive_path
        .ok_or("usage: mavc play [-r|--random | -o|--one <track-id>] <archive.mavc>")?;
    let archive = Archive::open(archive_path)?;
    let track = if let Some(id) = selected_id {
        archive
            .index
            .tracks
            .iter()
            .find(|track| track.id == id)
            .ok_or_else(|| format!("no track with id {id}"))?
    } else {
        archive
            .pick_random()
            .ok_or("no track has a positive random-play weight")?
    };
    let track_id = track.id;
    let title = track.title.clone();
    let codec = track.codec.clone();
    let length = track.length;
    if use_browser_player {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temp_dir = std::env::current_dir()?
            .join(format!("mavc-browser-{}-{timestamp}", std::process::id()));
        std::fs::create_dir(&temp_dir)?;
        let audio_path = temp_dir.join(format!("track.{codec}"));
        let html_path = temp_dir.join("player.html");
        archive.extract_track(archive_path, track_id, &audio_path)?;
        let html = format!(
            "<!doctype html><meta charset=\"utf-8\"><title>MAVC player</title>\
             <h1>MAVC: {}</h1><audio controls autoplay src=\"track.{}\"></audio>",
            escape_html(&title),
            codec
        );
        File::create(&html_path)?.write_all(html.as_bytes())?;
        open_in_default_browser(&html_path)?;
        println!("Opened track {track_id} in the default browser: {title}");
        println!("Browser page: {}", html_path.display());
        return Ok(());
    }
    if use_system_player {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temp_path = std::env::temp_dir().join(format!(
            "mavc-play-{}-{timestamp}.{codec}",
            std::process::id()
        ));
        archive.extract_track(archive_path, track_id, &temp_path)?;
        open_with_system_player(&temp_path)?;
        println!("Opened track {track_id}: {title}");
        println!("Temporary audio file: {}", temp_path.display());
        return Ok(());
    }
    let reader = archive.open_track(archive_path, track_id)?;
    let source = Decoder::builder()
        .with_data(reader)
        .with_byte_len(length)
        .with_hint(&codec)
        .with_gapless(true)
        .build()?;
    let output = DeviceSinkBuilder::open_default_sink()?;
    let player = Player::connect_new(&output.mixer());
    player.append(source);
    println!("Playing track {track_id}: {title}");
    player.sleep_until_end();
    Ok(())
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(target_os = "macos")]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_in_default_browser(path: &Path) -> std::io::Result<()> {
    Command::new("open").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_in_default_browser(path: &Path) -> std::io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_with_system_player(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_default_browser(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open").arg(path).spawn().map(|_| ())
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_with_system_player(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "opening audio with the system player is unsupported on this platform",
    ))
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open_in_default_browser(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "opening a browser is unsupported on this platform",
    ))
}

fn create_from_manifest(
    manifest_arg: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(manifest_arg);
    let mut bytes = Vec::new();
    File::open(&manifest_path)?.read_to_end(&mut bytes)?;
    let mut manifest: CreateManifest = serde_json::from_slice(&bytes)?;
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    for track in &mut manifest.tracks {
        if track.path.is_relative() {
            track.path = base.join(&track.path);
        }
    }
    create_archive(&manifest, output)?;
    println!("Created {output} with {} tracks", manifest.tracks.len());
    Ok(())
}

fn create_from_music_list(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut i = 0;
    while i < args.len() {
        if matches!(args[i].as_str(), "-m" | "--music-list") {
            // The marker can precede or follow the output option.
        } else if matches!(args[i].as_str(), "-a" | "--as") {
            i += 1;
            let name = args.get(i).ok_or("-a/--as needs an output name")?;
            output = Some(PathBuf::from(name));
        } else if args[i].starts_with('-') {
            return Err(format!("unknown option '{}'", args[i]).into());
        } else {
            inputs.push(PathBuf::from(&args[i]));
        }
        i += 1;
    }
    if inputs.is_empty() {
        return Err("-m/--music-list needs at least one audio file".into());
    }

    let output = output.unwrap_or_else(|| {
        let stem = if inputs.len() == 1 {
            inputs[0]
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("music")
        } else {
            "music"
        };
        PathBuf::from(format!("{stem}.mavc"))
    });
    let title = output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("music")
        .to_owned();
    let tracks = inputs
        .into_iter()
        .map(|path| {
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_owned();
            TrackSource {
                path,
                title,
                weight: 1.0,
            }
        })
        .collect::<Vec<_>>();
    let manifest = CreateManifest {
        describe: Description {
            title,
            creator: None,
            description: None,
        },
        tracks,
    };
    create_archive(&manifest, &output)?;
    println!(
        "Created {} with {} tracks",
        output.display(),
        manifest.tracks.len()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
