//! Validated access to a MAVC description, index, and its audio payload.
use super::error::Error;
use super::format::{HEADER_LEN, MAGIC, VERSION};
use super::metadata::metadata_from_tagged_file;
use super::model::AudioMetadata;
use super::model::{Description, MusicIndex, Track};
use lofty::file::FileType;
use lofty::probe::Probe;
use rand::Rng;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// An opened archive. Track offsets are relative to its audio payload section.
#[derive(Debug, Clone)]
pub struct Archive {
    pub describe: Description,
    pub index: MusicIndex,
    pub(super) payload_start: u64,
}

impl Archive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        // The first 22 bytes are fixed-size fields; the remaining section sizes
        // are read from this header before allocating their buffers.
        let mut fixed = [0u8; HEADER_LEN as usize];
        file.read_exact(&mut fixed)?;
        if &fixed[..4] != MAGIC {
            return Err(Error::Invalid("bad magic; expected MAVC".into()));
        }
        // Header layout: magic[0..4], version[4..6], description length[6..10],
        // index length[10..14], and payload length[14..22]. All integers are LE.
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
        // Track offsets are relative to this point, the beginning of audio bytes.
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
