use mavc::{Archive, AudioMetadata};

pub(super) fn print_track_list(archive: &Archive) {
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

pub(super) fn inspect_archive(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
