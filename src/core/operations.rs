use super::archive::Archive;
use super::config::load_limits;
use super::error::Error;
use super::format::{MAGIC, VERSION, replace_file, sibling_temp_path};
use super::metadata::{codec_for, read_audio_metadata};
use super::model::{CreateManifest, MusicIndex, Track};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};

/// Write a MAVC archive from a JSON manifest. Track offsets in `mvlc.json` are
/// relative to the beginning of the audio payload area.
pub fn create_archive(manifest: &CreateManifest, output: impl AsRef<Path>) -> Result<(), Error> {
    if manifest.tracks.is_empty() {
        return Err(Error::Invalid("at least one track is required".into()));
    }
    let mut tracks = Vec::with_capacity(manifest.tracks.len());
    let mut payload_len = 0u64;
    for (id, source) in manifest.tracks.iter().enumerate() {
        if !source.weight.is_finite() || source.weight < 0.0 {
            return Err(Error::Invalid(format!(
                "track '{}' has an invalid weight",
                source.title
            )));
        }
        let codec = codec_for(&source.path)?;
        let length = std::fs::metadata(&source.path)?.len();
        let file_name = source
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Invalid("track path must have a UTF-8 file name".into()))?
            .to_owned();
        tracks.push(Track {
            id,
            title: source.title.clone(),
            file_name,
            codec: codec.into(),
            offset: payload_len,
            length,
            weight: source.weight,
            metadata: read_audio_metadata(&source.path),
        });
        payload_len = payload_len
            .checked_add(length)
            .ok_or_else(|| Error::Invalid("archive payload is too large".into()))?;
    }
    let track_lengths: Vec<u64> = tracks.iter().map(|track| track.length).collect();
    // The index stores metadata and payload-relative offsets; the audio bytes
    // themselves are streamed after the two JSON sections.
    let describe = serde_json::to_vec(&manifest.describe)?;
    let index = serde_json::to_vec(&MusicIndex {
        format: "MVLC".into(),
        version: VERSION,
        tracks,
    })?;
    let describe_len = u32::try_from(describe.len())
        .map_err(|_| Error::Invalid("description is too large".into()))?;
    let index_len =
        u32::try_from(index.len()).map_err(|_| Error::Invalid("index is too large".into()))?;

    // Emit the fixed header, then both JSON sections, then each source file in
    // the same order used to assign track offsets above.
    let mut out = File::create(output)?;
    out.write_all(MAGIC)?;
    out.write_all(&VERSION.to_le_bytes())?;
    out.write_all(&describe_len.to_le_bytes())?;
    out.write_all(&index_len.to_le_bytes())?;
    out.write_all(&payload_len.to_le_bytes())?;
    out.write_all(&describe)?;
    out.write_all(&index)?;
    for (position, source) in manifest.tracks.iter().enumerate() {
        let mut input = File::open(&source.path)?;
        let copied = io::copy(&mut input, &mut out)?;
        if copied != track_lengths[position] {
            return Err(Error::Invalid(format!(
                "{} changed while packaging",
                source.path.display()
            )));
        }
    }
    Ok(())
}

/// Add audio files to each named archive, preserving its description and tracks.
pub fn add_tracks(archive_paths: &[PathBuf], audio_paths: &[PathBuf]) -> Result<(), Error> {
    if archive_paths.is_empty() {
        return Err(Error::Invalid("add requires at least one archive".into()));
    }
    if audio_paths.is_empty() {
        return Err(Error::Invalid(
            "add requires at least one audio file".into(),
        ));
    }
    for path in archive_paths {
        rewrite_archive(path, audio_paths, &[])?;
    }
    Ok(())
}

/// Remove selected track IDs from each named archive.
pub fn remove_tracks(archive_paths: &[PathBuf], track_ids: &[usize]) -> Result<(), Error> {
    if archive_paths.is_empty() {
        return Err(Error::Invalid(
            "remove requires at least one archive".into(),
        ));
    }
    let ids: std::collections::HashSet<usize> = track_ids.iter().copied().collect();
    let ids: Vec<usize> = ids.into_iter().collect();
    for path in archive_paths {
        rewrite_archive(path, &[], &ids)?;
    }
    Ok(())
}

fn rewrite_archive(path: &Path, additions: &[PathBuf], removals: &[usize]) -> Result<(), Error> {
    let limits = load_limits()?;
    let max_payload_bytes = limits
        .max_size_mb
        .checked_mul(1_000_000)
        .ok_or_else(|| Error::Invalid("limits.max_size_mb is too large".into()))?;
    let archive = Archive::open(path)?;
    let mut tracks = archive.index.tracks.clone();
    for id in removals {
        if !tracks.iter().any(|track| track.id == *id) {
            return Err(Error::Invalid(format!(
                "no track with id {id} in {}",
                path.display()
            )));
        }
    }
    tracks.retain(|track| !removals.contains(&track.id));
    let mut payload_len = tracks.iter().map(|track| track.length).sum::<u64>();
    let final_track_count = tracks
        .len()
        .checked_add(additions.len())
        .ok_or_else(|| Error::Invalid("too many tracks".into()))?;
    if final_track_count > limits.max_tracks {
        return Err(Error::Invalid(format!(
            "{} would contain {final_track_count} tracks; the configured limit is {}",
            path.display(),
            limits.max_tracks
        )));
    }
    for source in additions {
        let codec = codec_for(source)?;
        let length = std::fs::metadata(source)?.len();
        if payload_len
            .checked_add(length)
            .is_none_or(|size| size > max_payload_bytes)
        {
            return Err(Error::Invalid(format!(
                "{} would exceed the configured {} MB audio payload limit",
                path.display(),
                limits.max_size_mb
            )));
        }
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Invalid("track path must have a UTF-8 file name".into()))?
            .to_owned();
        let id = (0..)
            .find(|candidate| !tracks.iter().any(|track| track.id == *candidate))
            .ok_or_else(|| Error::Invalid("no track IDs available".into()))?;
        tracks.push(Track {
            id,
            title: source
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_name)
                .to_owned(),
            file_name,
            codec: codec.into(),
            offset: payload_len,
            length,
            weight: 1.0,
            metadata: read_audio_metadata(source),
        });
        payload_len = payload_len
            .checked_add(length)
            .ok_or_else(|| Error::Invalid("archive payload is too large".into()))?;
    }
    // Compact offsets and preserve existing payload bytes in track order.
    let mut offset = 0u64;
    for track in &mut tracks {
        track.offset = offset;
        offset = offset
            .checked_add(track.length)
            .ok_or_else(|| Error::Invalid("archive payload is too large".into()))?;
    }
    let description = serde_json::to_vec(&archive.describe)?;
    let index = serde_json::to_vec(&MusicIndex {
        format: "MVLC".into(),
        version: VERSION,
        tracks: tracks.clone(),
    })?;
    let description_len = u32::try_from(description.len())
        .map_err(|_| Error::Invalid("description is too large".into()))?;
    let index_len =
        u32::try_from(index.len()).map_err(|_| Error::Invalid("index is too large".into()))?;
    // Write a complete replacement beside the original first. On failure the
    // original remains intact; successful writes are atomically renamed below.
    let temp = sibling_temp_path(path);
    let result = (|| -> Result<(), Error> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        // Rebuild the header because removals/additions change section lengths.
        output.write_all(MAGIC)?;
        output.write_all(&VERSION.to_le_bytes())?;
        output.write_all(&description_len.to_le_bytes())?;
        output.write_all(&index_len.to_le_bytes())?;
        output.write_all(&offset.to_le_bytes())?;
        output.write_all(&description)?;
        output.write_all(&index)?;
        let mut input = File::open(path)?;
        for track in archive
            .index
            .tracks
            .iter()
            .filter(|track| !removals.contains(&track.id))
        {
            input.seek(SeekFrom::Start(archive.payload_start + track.offset))?;
            io::copy(
                &mut Read::by_ref(&mut input).take(track.length),
                &mut output,
            )?;
        }
        for source in additions {
            io::copy(&mut File::open(source)?, &mut output)?;
        }
        output.sync_all()?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    replace_file(&temp, path)
}

/// Validate a user-supplied relative path before using it as an extraction target.
pub fn safe_relative_path(path: &str) -> Result<PathBuf, Error> {
    let p = Path::new(path);
    if p.as_os_str().is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(Error::Invalid("unsafe archive path".into()));
    }
    Ok(p.to_path_buf())
}

impl Archive {
    /// Update a track's selection weight and rewrite the archive index while
    /// preserving the packed audio payload.
    pub fn update_track_weight(
        archive_path: impl AsRef<Path>,
        track_id: usize,
        weight: f64,
    ) -> Result<(f64, f64), Error> {
        if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
            return Err(Error::Invalid(
                "weight must be a finite number between 0 and 1".into(),
            ));
        }

        let archive_path = archive_path.as_ref();
        let archive = Self::open(archive_path)?;
        let mut updated_index = archive.index.clone();
        let track = updated_index
            .tracks
            .iter_mut()
            .find(|track| track.id == track_id)
            .ok_or_else(|| Error::Invalid(format!("no track with id {track_id}")))?;
        let old_weight = track.weight;
        track.weight = weight;
        let other_track_count = updated_index.tracks.len().saturating_sub(1);
        if other_track_count > 0 {
            let other_weight = (1.0 - weight) / other_track_count as f64;
            for track in &mut updated_index.tracks {
                if track.id != track_id {
                    track.weight = other_weight;
                }
            }
        }
        let new_weight = updated_index
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .expect("updated track remains in index")
            .weight;

        let description = serde_json::to_vec(&archive.describe)?;
        let index = serde_json::to_vec(&updated_index)?;
        let description_len = u32::try_from(description.len())
            .map_err(|_| Error::Invalid("description is too large".into()))?;
        let index_len =
            u32::try_from(index.len()).map_err(|_| Error::Invalid("index is too large".into()))?;

        let mut source = File::open(archive_path)?;
        let payload_len = source.metadata()?.len() - archive.payload_start;
        let temp_path = sibling_temp_path(archive_path);
        let write_result = (|| -> Result<(), Error> {
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)?;
            output.write_all(MAGIC)?;
            output.write_all(&VERSION.to_le_bytes())?;
            output.write_all(&description_len.to_le_bytes())?;
            output.write_all(&index_len.to_le_bytes())?;
            output.write_all(&payload_len.to_le_bytes())?;
            output.write_all(&description)?;
            output.write_all(&index)?;
            source.seek(SeekFrom::Start(archive.payload_start))?;
            io::copy(&mut source, &mut output)?;
            output.sync_all()?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = std::fs::remove_file(&temp_path);
            return Err(error);
        }

        replace_file(&temp_path, archive_path)?;
        Ok((old_weight, new_weight))
    }
}
