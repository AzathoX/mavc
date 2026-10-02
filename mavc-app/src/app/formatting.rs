//! Shared helpers for track labels, durations, and optional form values.

use super::models::Track;

pub(super) fn display_title(track: &Track) -> &str {
    track
        .metadata
        .title
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(&track.title)
}

pub(super) fn format_time_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "--:--".into();
    }
    let seconds = seconds.floor() as u64;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub(super) fn optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}
