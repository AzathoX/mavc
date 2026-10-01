//! Constants and filesystem helpers shared by the MAVC format readers and writers.
//!
//! Version 1 stores a fixed-size, little-endian header followed by a description,
//! an MVLC index, and the concatenated audio payload.
use super::error::Error;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Four-byte signature at the start of every archive.
pub(super) const MAGIC: &[u8; 4] = b"MAVC";
/// Version of the outer MAVC container format.
pub(super) const VERSION: u16 = 1;
/// Header size: 4 + 2 + 4 + 4 + 8 bytes.
pub(super) const HEADER_LEN: u64 = 22;

pub(super) fn sibling_temp_path(path: &Path) -> PathBuf {
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
pub(super) fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), Error> {
    std::fs::rename(temp_path, destination)?;
    Ok(())
}

#[cfg(windows)]
pub(super) fn replace_file(temp_path: &Path, destination: &Path) -> Result<(), Error> {
    let backup_path = sibling_temp_path(destination).with_extension("bak");
    std::fs::rename(destination, &backup_path)?;
    if let Err(error) = std::fs::rename(temp_path, destination) {
        let _ = std::fs::rename(&backup_path, destination);
        return Err(Error::Io(error));
    }
    std::fs::remove_file(backup_path)?;
    Ok(())
}
