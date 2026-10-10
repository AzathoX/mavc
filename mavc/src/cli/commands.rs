use super::args::{Commands, DoAction};
use mavc::{Archive, CreateManifest, add_tracks, create_archive, music, remove_tracks};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) fn run(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Do(args) => {
            return run(match args.action {
                DoAction::Add(args) => Commands::Add(args),
                DoAction::Remove(args) => Commands::Remove(args),
            });
        }
        Commands::Package(args) => {
            if !args.files.is_empty() {
                if args.manifest.is_some() || args.manifest_output.is_some() {
                    return Err("package music-list mode does not accept manifest paths".into());
                }
                create_from_music_list(args.files, args.output)?;
            } else if let (Some(manifest), Some(output)) = (args.manifest, args.manifest_output) {
                create_from_manifest(&manifest, &output)?;
            } else {
                return Err("package expects -m audio files or a manifest and output path".into());
            }
        }
        Commands::Music(args) => {
            create_from_music_list(args.files, args.output)?;
        }
        Commands::MusicShortcut(args) => {
            create_from_music_list(args.files, args.output)?;
        }
        Commands::Create(args) => create_from_manifest(&args.manifest, &args.output)?,
        Commands::Add(args) => {
            add_tracks(&args.archives, &args.files)?;
            for path in args.archives {
                println!("Updated {}", path.display());
            }
        }
        Commands::Remove(args) => {
            export_before_remove(&args.archives, &args.ids, &args.outputs)?;
            remove_tracks(&args.archives, &args.ids)?;
            for path in args.archives {
                println!("Updated {}", path.display());
            }
        }
        Commands::List(args) => {
            let archive = Archive::open(&args.archive)?;
            super::output::print_track_list(&archive);
        }
        Commands::Inspect(args) => {
            let values = std::iter::once(args.archive_or_id)
                .chain(args.second)
                .collect::<Vec<_>>();
            super::output::inspect_archive(&values)?;
        }
        Commands::Pick(args) => {
            let archive = Archive::open(&args.archive)?;
            match archive.pick_random() {
                Some(track) => println!("{}\t{}\t{}", track.id, track.title, track.file_name),
                None => return Err("no track has a positive random-play weight".into()),
            }
        }
        Commands::Weight(args) => {
            let (track_id, weight) = args
                .assignment
                .split_once('=')
                .ok_or("weight assignment must use <track-id>=<weight>")?;
            let track_id: usize = track_id.parse()?;
            let weight: f64 = weight.parse()?;
            let (old_weight, new_weight) =
                Archive::update_track_weight(&args.archive, track_id, weight)?;
            println!("The weight update succeeded: from {old_weight} to {new_weight}.");
        }
        Commands::Extract(args) => {
            let archive = Archive::open(&args.archive)?;
            if args.all {
                if archive.index.tracks.is_empty() {
                    return Err("archive has no tracks to extract".into());
                }
                let current_dir = std::env::current_dir()?;
                let mut outputs = std::collections::HashSet::new();
                for track in &archive.index.tracks {
                    let name = Path::new(&track.file_name)
                        .file_name()
                        .ok_or("track has no usable filename")?;
                    let collision_key = name.to_string_lossy().to_lowercase();
                    if !outputs.insert(collision_key) {
                        return Err(format!(
                            "multiple tracks have the filename {}; cannot extract all without overwriting",
                            name.to_string_lossy()
                        )
                        .into());
                    }
                    let output = current_dir.join(name);
                    if output.exists() {
                        return Err(format!(
                            "{} already exists; refusing to overwrite it",
                            output.display()
                        )
                        .into());
                    }
                }
                for track in &archive.index.tracks {
                    let name = Path::new(&track.file_name)
                        .file_name()
                        .ok_or("track has no usable filename")?;
                    let output = current_dir.join(name);
                    archive.extract_track(&args.archive, track.id, &output)?;
                    println!("Extracted track {} to {}", track.id, output.display());
                }
            } else {
                let track_id = args.track_id.expect("clap requires track_id unless --all");
                let output = match args.output {
                    Some(path) => path,
                    None => {
                        let track = archive
                            .index
                            .tracks
                            .iter()
                            .find(|track| track.id == track_id)
                            .ok_or_else(|| format!("no track with id {track_id}"))?;
                        PathBuf::from(
                            Path::new(&track.file_name)
                                .file_name()
                                .ok_or("track has no usable filename")?,
                        )
                    }
                };
                archive.extract_track(&args.archive, track_id, &output)?;
                println!("Extracted track {track_id} to {}", output.display());
            }
        }
        Commands::Play(args) => super::player::play_random(args)?,
    }
    Ok(())
}

fn create_from_manifest(
    manifest_path: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    File::open(&manifest_path)?.read_to_end(&mut bytes)?;
    let mut manifest: CreateManifest = serde_json::from_slice(&bytes)?;
    let base = manifest_path.parent().unwrap_or(Path::new("../../.."));
    for track in &mut manifest.tracks {
        if track.path.is_relative() {
            track.path = base.join(&track.path);
        }
    }
    create_archive(&manifest, output)?;
    println!(
        "Created {} with {} tracks",
        output.display(),
        manifest.tracks.len()
    );
    Ok(())
}

fn create_from_music_list(
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let track_count = inputs.len();
    let mut builder = music().from(inputs);
    if let Some(output) = output {
        builder = builder.output(output);
    }
    let output = builder.create()?;
    println!("Created {} with {track_count} tracks", output.display());
    Ok(())
}

fn export_before_remove(
    archives: &[PathBuf],
    ids: &[usize],
    directories: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    if directories.is_empty() {
        return Ok(());
    }
    let mut jobs = Vec::new();
    let mut targets = std::collections::HashSet::new();
    // Validate the complete export plan before writing or deleting any songs.
    for path in archives {
        let archive = Archive::open(path)?;
        let mut seen_ids = std::collections::HashSet::new();
        for id in ids {
            if !seen_ids.insert(*id) {
                continue;
            }
            let track = archive
                .index
                .tracks
                .iter()
                .find(|track| track.id == *id)
                .ok_or_else(|| format!("no track with id {id} in {}", path.display()))?;
            let filename = Path::new(&track.file_name)
                .file_name()
                .filter(|name| !name.is_empty())
                .ok_or("track has no usable filename")?;
            for directory in directories {
                std::fs::create_dir_all(directory)?;
                let target = std::fs::canonicalize(directory)?.join(filename);
                if target.exists() || !targets.insert(target.to_string_lossy().to_lowercase()) {
                    return Err(format!(
                        "{} already exists or has a duplicate export filename",
                        target.display()
                    )
                    .into());
                }
                jobs.push((archive.clone(), path.clone(), *id, target, track.length));
            }
        }
    }
    for (archive, source, id, target, length) in jobs {
        let mut input = archive.open_track(&source, id)?;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        let result = (|| -> std::io::Result<()> {
            if std::io::copy(&mut input, &mut output)? != length {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "incomplete track export",
                ));
            }
            output.sync_all()
        })();
        if let Err(error) = result {
            drop(output);
            let _ = std::fs::remove_file(&target);
            return Err(error.into());
        }
        println!("Exported track {id} to {}", target.display());
    }
    Ok(())
}
