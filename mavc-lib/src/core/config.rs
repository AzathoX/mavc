//! Runtime limits used when existing archives are rewritten.
use super::error::Error;
use serde::Deserialize;
use std::io;
use std::path::Path;

const DEFAULT_MAX_TRACKS: usize = 5;
const DEFAULT_MAX_SIZE_MB: u64 = 150;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    #[serde(default)]
    limits: Limits,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Limits {
    #[serde(default = "default_max_tracks")]
    pub(super) max_tracks: usize,
    #[serde(default = "default_max_size_mb")]
    pub(super) max_size_mb: u64,
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

pub(super) fn load_limits() -> Result<Limits, Error> {
    let config_path = Path::new("../../../mavc.toml");
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
