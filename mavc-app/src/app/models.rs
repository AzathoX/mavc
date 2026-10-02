//! Frontend data transfer types returned by the desktop command layer.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveInfo {
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) creator: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) tracks: Vec<Track>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Track {
    pub(crate) id: usize,
    pub(crate) title: String,
    pub(crate) file_name: String,
    pub(crate) weight: f64,
    pub(crate) cover_art_url: Option<String>,
    #[serde(default)]
    pub(crate) embedded_cover_data_url: Option<String>,
    pub(crate) metadata: Metadata,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) title: Option<String>,
    pub(crate) artist: Option<String>,
    pub(crate) duration_ms: Option<u64>,
    pub(crate) album: Option<String>,
    pub(crate) genre: Option<String>,
    pub(crate) year: Option<String>,
    pub(crate) bitrate_kbps: Option<u32>,
    pub(crate) sample_rate_hz: Option<u32>,
    pub(crate) channels: Option<u8>,
    pub(crate) bit_depth: Option<u8>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlaybackData {
    pub(crate) archive_title: String,
    pub(crate) track: Track,
    pub(crate) src: String,
}

#[derive(Clone, Debug)]
pub(crate) struct MixedSource {
    pub(crate) id: usize,
    pub(crate) title: String,
    pub(crate) src: String,
    pub(crate) position_seconds: f64,
    pub(crate) duration_seconds: f64,
}
