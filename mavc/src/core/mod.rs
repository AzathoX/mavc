//! MAVC archive implementation, kept behind the stable crate-level re-exports.
//!
//! `model` defines serialized data, `format` defines container bytes, and the
//! remaining modules handle reading, writing, metadata, limits, and errors.
mod archive;
mod config;
mod error;
mod format;
mod metadata;
mod model;
mod operations;

pub use archive::Archive;
pub use error::Error;
pub use model::{AudioMetadata, CreateManifest, Description, MusicIndex, Track, TrackSource};
pub use operations::{add_tracks, create_archive, remove_tracks, safe_relative_path};
