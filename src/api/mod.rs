use crate::{Archive, CreateManifest, Description, Error, Track, TrackSource, create_archive};
use std::path::{Path, PathBuf};

/// Build a MAVC archive from a list of audio files.
///
/// # Example
///
/// ```no_run
/// mavc::music()
///     .from(["song-a.mp3", "song-b.flac"])
///     .output("collection.mavc")
///     .create()?;
/// # Ok::<(), mavc::Error>(())
/// ```
pub fn music() -> MusicBuilder {
    MusicBuilder::default()
}

/// Builder for creating an archive from audio files with equal initial weights.
#[derive(Debug, Default)]
pub struct MusicBuilder {
    files: Vec<PathBuf>,
    output: Option<PathBuf>,
}

/// Start a playback selection and return the selected track as a byte stream.
///
/// Random weighted selection is the default. Use [`PlayBuilder::one`] to select
/// a track ID explicitly.
///
/// # Example
///
/// ```no_run
/// let selected = mavc::play()
///     .archive("collection.mavc")
///     .one(2)
///     .stream()?;
/// let track_title = selected.track.title;
/// let audio_bytes = selected.bytes;
/// # let _ = (track_title, audio_bytes);
/// # Ok::<(), mavc::Error>(())
/// ```
pub fn play() -> PlayBuilder {
    PlayBuilder::default()
}

/// Builder for selecting an archive track and opening its bytes for streaming.
#[derive(Debug, Default)]
pub struct PlayBuilder {
    archive: Option<PathBuf>,
    track_id: Option<usize>,
}

/// A selected track and its length-limited audio byte stream.
#[derive(Debug)]
pub struct PlaybackTrack {
    /// Metadata for the selected track.
    pub track: Track,
    /// Reads only the selected track's bytes from the archive.
    pub bytes: std::io::Take<std::fs::File>,
}

impl PlayBuilder {
    /// Set the archive to read from.
    pub fn archive(mut self, path: impl Into<PathBuf>) -> Self {
        self.archive = Some(path.into());
        self
    }

    /// Use weighted random selection (the default).
    pub fn random(mut self) -> Self {
        self.track_id = None;
        self
    }

    /// Select a track by its ID instead of choosing randomly.
    pub fn one(mut self, track_id: usize) -> Self {
        self.track_id = Some(track_id);
        self
    }

    /// Open the selected track and return its metadata with a byte stream.
    pub fn stream(self) -> Result<PlaybackTrack, Error> {
        let path = self
            .archive
            .ok_or_else(|| Error::Invalid("an archive path is required".into()))?;
        let archive = Archive::open(&path)?;
        let track_id = match self.track_id {
            Some(track_id) => track_id,
            None => archive.pick_random().map(|track| track.id).ok_or_else(|| {
                Error::Invalid("no track has a positive random-play weight".into())
            })?,
        };
        let track = archive
            .index
            .tracks
            .iter()
            .find(|track| track.id == track_id)
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("no track with id {track_id}")))?;
        let bytes = archive.open_track(&path, track_id)?;
        Ok(PlaybackTrack { track, bytes })
    }
}

impl MusicBuilder {
    /// Set the audio files to include in the archive.
    pub fn from<I, P>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Set the output archive path. If omitted, MAVC derives a name from the input files.
    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Some(path.into());
        self
    }

    /// Create the archive and return its path.
    pub fn create(self) -> Result<PathBuf, Error> {
        if self.files.is_empty() {
            return Err(Error::Invalid("at least one audio file is required".into()));
        }

        let output = self
            .output
            .unwrap_or_else(|| default_output_path(&self.files));
        let title = output
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("music")
            .to_owned();
        let initial_weight = 1.0 / self.files.len() as f64;
        let tracks = self
            .files
            .into_iter()
            .map(|path| {
                let title = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("untitled")
                    .to_owned();
                TrackSource {
                    path,
                    title,
                    weight: initial_weight,
                }
            })
            .collect();
        let manifest = CreateManifest {
            describe: Description {
                title,
                creator: None,
                description: None,
            },
            tracks,
        };

        create_archive(&manifest, &output)?;
        Ok(output)
    }
}

fn default_output_path(files: &[PathBuf]) -> PathBuf {
    let stem = if files.len() == 1 {
        files[0]
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("music")
    } else {
        "music"
    };
    Path::new(&format!("{stem}.mavc")).to_path_buf()
}
