//! Audio codec checks and tag/property extraction used when indexing tracks.
use super::error::Error;
use super::model::AudioMetadata;
use lofty::file::{AudioFile, TaggedFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::Path;

pub(super) fn codec_for(path: &Path) -> Result<&'static str, Error> {
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

pub(super) fn read_audio_metadata(path: &Path) -> AudioMetadata {
    let Ok(probe) = Probe::open(path) else {
        return AudioMetadata::default();
    };
    let Ok(tagged_file) = probe.read() else {
        return AudioMetadata::default();
    };
    metadata_from_tagged_file(&tagged_file)
}

pub(super) fn metadata_from_tagged_file(tagged_file: &TaggedFile) -> AudioMetadata {
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
