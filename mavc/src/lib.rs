//! MAVC archive library. The public API is re-exported here while its
//! implementation is organized under the private `core` module.
/// High-level builder APIs for common archive creation flows.
pub mod api;
// Keep internal modules private while preserving the crate's public API below.
mod core;

pub use api::{MusicBuilder, PlayBuilder, PlaybackTrack, music, play};

pub use core::{
    Archive, AudioMetadata, CreateManifest, Description, Error, MusicIndex, Track, TrackSource,
    add_tracks, create_archive, remove_tracks, safe_relative_path,
};

/// Version of the linked MAVC library (independent of desktop app versions).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// File extensions accepted by the library's import validator.
pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "aac", "wav", "ogg"];
