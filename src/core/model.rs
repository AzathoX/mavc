//! Serializable descriptions and track/index records stored in MAVC archives.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Human-readable description stored before the MVLC index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description {
    pub title: String,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// An input file and its title/weight before it is packed into an archive.
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

/// Complete input description consumed by [`crate::create_archive`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateManifest {
    pub describe: Description,
    pub tracks: Vec<TrackSource>,
}

/// One audio track's metadata and payload-relative byte range in an archive.
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

/// Audio tags and technical properties cached in each index track.
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

/// JSON index stored between the archive description and audio payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicIndex {
    pub format: String,
    pub version: u16,
    pub tracks: Vec<Track>,
}
