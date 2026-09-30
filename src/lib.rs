use lofty::file::{AudioFile, FileType, TaggedFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 4] = b"MAVC";
const VERSION: u16 = 1;
const HEADER_LEN: u64 = 22;
const DEFAULT_MAX_TRACKS: usize = 5;
const DEFAULT_MAX_SIZE_MB: u64 = 150;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default)]
    limits: Limits,
}

#[derive(Debug, Clone, Deserialize)]
struct Limits {
    #[serde(default = "default_max_tracks")]
    max_tracks: usize,
    #[serde(default = "default_max_size_mb")]
    max_size_mb: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_tracks: default_max_tracks(),
            max_size_mb: default_max_size_mb(),
        }
    }
}

fn default_max_tracks() -> usize {
    DEFAULT_MAX_TRACKS
}
fn default_max_size_mb() -> u64 {
    DEFAULT_MAX_SIZE_MB
}

fn load_limits() -> Result<Limits, Error> {
    let config_path = Path::new("mavc.toml");
    let config = match std::fs::read_to_string(config_path) {
        Ok(contents) => toml::from_str::<Config>(&contents)
            .map_err(|error| Error::Invalid(format!("{}: {error}", config_path.display())))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => Config {
            limits: Limits::default(),
        },
        Err(error) => return Err(error.into()),
    };
    if config.limits.max_tracks == 0 || config.limits.max_size_mb == 0 {
        return Err(Error::Invalid(
            "mavc.toml limits.max_tracks and limits.max_size_mb must be greater than zero".into(),
        ));
    }
    Ok(config.limits)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub title: String,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSource {
    pub path: PathBuf,
    pub title: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateManifest {
    pub describe: Description,
    pub tracks: Vec<TrackSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub file_name: String,
    pub codec: String,
    pub offset: u64,
    pub length: u64,
    pub weight: f64,
    #[serde(default)]
    pub metadata: AudioMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioMetadata {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub track_number: Option<u32>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub bitrate_kbps: Option<u32>,
    #[serde(default)]
    pub sample_rate_hz: Option<u32>,
    #[serde(default)]
    pub channels: Option<u8>,
    #[serde(default)]
    pub bit_depth: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicIndex {
    pub format: String,
    pub version: u16,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Archive {
    pub describe: Description,
    pub index: MusicIndex,
    payload_start: u64,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Invalid(s) => write!(f, "invalid MAVC archive: {s}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

fn codec_for(path: &Path) -> Result<&'static str, Error> {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mp3" => Ok("mp3"),
        "flac" => Ok("flac"),
        "aac" => Ok("aac"),
        ext => Err(Error::Invalid(format!(
            "unsupported audio extension '.{ext}' (use MP3, FLAC, or AAC)"
        ))),
    }
}

fn read_audio_metadata(path: &Path) -> AudioMetadata {
    let Ok(probe) = Probe::open(path) else {
        return AudioMetadata::default();
    };
    let Ok(tagged_file) = probe.read() else {
        return AudioMetadata::default();
    };
    metadata_from_tagged_file(&tagged_file)
}

fn metadata_from_tagged_file(tagged_file: &TaggedFile) -> AudioMetadata {
    let properties = tagged_file.properties();
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    AudioMetadata {
        title: tag.and_then(|tag| tag.title().map(|s| s.into_owned())),
        artist: tag.and_then(|tag| tag.artist().map(|s| s.into_owned())),
        album: tag.and_then(|tag| tag.album().map(|s| s.into_owned())),
        genre: tag.and_then(|tag| tag.genre().map(|s| s.into_owned())),
        year: tag.and_then(|tag| tag.date().map(|date| date.to_string())),
        track_number: tag.and_then(|tag| tag.track()),
        comment: tag.and_then(|tag| tag.comment().map(|s| s.into_owned())),
        duration_ms: Some(properties.duration().as_millis().min(u64::MAX as u128) as u64),
        bitrate_kbps: properties.audio_bitrate(),
        sample_rate_hz: properties.sample_rate(),
        channels: properties.channels(),
        bit_depth: properties.bit_depth(),
    }
}

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
    if ids.len() < 2 {
        return Err(Error::Invalid(
            "remove requires at least 2 track IDs".into(),
        ));
    }
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
    let temp = sibling_temp_path(path);
    let result = (|| -> Result<(), Error> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
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

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut fixed = [0u8; HEADER_LEN as usize];
        file.read_exact(&mut fixed)?;
        if &fixed[..4] != MAGIC {
            return Err(Error::Invalid("bad magic; expected MAVC".into()));
        }
        let version = u16::from_le_bytes(fixed[4..6].try_into().unwrap());
        if version != VERSION {
            return Err(Error::Invalid(format!(
                "unsupported container version {version}"
            )));
        }
        let dlen = u32::from_le_bytes(fixed[6..10].try_into().unwrap()) as usize;
        let ilen = u32::from_le_bytes(fixed[10..14].try_into().unwrap()) as usize;
        let plen = u64::from_le_bytes(fixed[14..22].try_into().unwrap());
        let mut d = vec![0; dlen];
        let mut i = vec![0; ilen];
        file.read_exact(&mut d)?;
        file.read_exact(&mut i)?;
        let describe: Description = serde_json::from_slice(&d)?;
        let index: MusicIndex = serde_json::from_slice(&i)?;
        if index.format != "MVLC" || index.version != VERSION {
            return Err(Error::Invalid("unsupported or malformed MVLC index".into()));
        }
        let payload_start = HEADER_LEN + dlen as u64 + ilen as u64;
        let expected_len = payload_start
            .checked_add(plen)
            .ok_or_else(|| Error::Invalid("archive length overflow".into()))?;
        if file.metadata()?.len() != expected_len {
            return Err(Error::Invalid(
                "payload length does not match archive size".into(),
            ));
        }
        let mut ranges = Vec::with_capacity(index.tracks.len());
        for track in &index.tracks {
            if !track.weight.is_finite() || track.weight < 0.0 {
                return Err(Error::Invalid("index contains an invalid weight".into()));
            }
            let end = track
                .offset
                .checked_add(track.length)
                .ok_or_else(|| Error::Invalid("track range overflow".into()))?;
            if end > plen {
                return Err(Error::Invalid(format!(
                    "track {} extends beyond payload",
                    track.id
                )));
            }
            ranges.push((track.offset, end));
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(Error::Invalid("track payload ranges overlap".into()));
        }
        Ok(Self {
            describe,
            index,
            payload_start,
        })
    }

    pub fn pick_random(&self) -> Option<&Track> {
        let total: f64 = self.index.tracks.iter().map(|t| t.weight).sum();
        if total <= 0.0 || !total.is_finite() {
            return None;
        }
        let mut choice = rand::thread_rng().gen_range(0.0..total);
        self.index.tracks.iter().find(|track| {
            if choice < track.weight {
                true
            } else {
                choice -= track.weight;
                false
            }
        })
    }

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

    pub fn extract_track(
        &self,
        archive_path: impl AsRef<Path>,
        track_id: usize,
        output: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let track = self
            .index
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .ok_or_else(|| Error::Invalid(format!("no track with id {track_id}")))?;
        let mut input = File::open(archive_path)?;
        input.seek(SeekFrom::Start(self.payload_start + track.offset))?;
        let mut limited = input.take(track.length);
        let mut out = File::create(output)?;
        io::copy(&mut limited, &mut out)?;
        Ok(())
    }

    /// Open a seekable, length-limited reader for one audio track in the archive.
    /// The returned reader streams bytes from the archive without extracting a copy.
    pub fn open_track(
        &self,
        archive_path: impl AsRef<Path>,
        track_id: usize,
    ) -> Result<io::Take<File>, Error> {
        let track = self
            .index
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .ok_or_else(|| Error::Invalid(format!("no track with id {track_id}")))?;
        let mut input = File::open(archive_path)?;
        input.seek(SeekFrom::Start(self.payload_start + track.offset))?;
        Ok(input.take(track.length))
    }

    /// Return embedded metadata, reading the audio payload if an older index has none.
    pub fn track_metadata(
        &self,
        archive_path: impl AsRef<Path>,
        track_id: usize,
    ) -> Result<AudioMetadata, Error> {
        let track = self
            .index
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .ok_or_else(|| Error::Invalid(format!("no track with id {track_id}")))?;
        if track.metadata.duration_ms.is_some()
            || track.metadata.title.is_some()
            || track.metadata.artist.is_some()
            || track.metadata.album.is_some()
        {
            return Ok(track.metadata.clone());
        }
        let file_type = match track.codec.as_str() {
            "mp3" => FileType::Mpeg,
            "flac" => FileType::Flac,
            "aac" => FileType::Aac,
            _ => return Ok(track.metadata.clone()),
        };
        let reader = self.open_track(archive_path, track_id)?;
        match Probe::with_file_type(reader, file_type).read() {
            Ok(tagged_file) => Ok(metadata_from_tagged_file(&tagged_file)),
            Err(_) => Ok(track.metadata.clone()),
        }
    }
}

fn sibling_temp_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    parent.join(format!(
        ".{name}.mavc-{}-{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

#[cfg(not(windows))]
fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), Error> {
    std::fs::rename(temp_path, destination)?;
    Ok(())
}

#[cfg(windows)]
fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), Error> {
    let backup_path = sibling_temp_path(destination).with_extension("bak");
    std::fs::rename(destination, &backup_path)?;
    if let Err(error) = std::fs::rename(temp_path, destination) {
        let _ = std::fs::rename(&backup_path, destination);
        return Err(Error::Io(error));
    }
    std::fs::remove_file(backup_path)?;
    Ok(())
}

/// Validate a user-supplied relative path before using it as an extraction target.
pub fn safe_relative_path(path: &str) -> Result<PathBuf, Error> {
    let p = Path::new(path);
    if p.as_os_str().is_empty() || p.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(Error::Invalid("unsafe archive path".into()));
    }
    Ok(p.to_path_buf())
}
