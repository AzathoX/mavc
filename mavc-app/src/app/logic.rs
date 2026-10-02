//! Shared application state and non-visual helpers for the MAVC desktop UI.

use dioxus::prelude::*;

pub(super) use super::{
    bridge::command,
    formatting::{display_title, format_time_seconds, optional},
    i18n::{Language, tr},
    models::{ArchiveInfo, MixedSource, PlaybackData, Track},
    playback::{seek_audio_to, seek_combined_track, set_audio_playing},
};

/// Holds the reactive values shared by archive, track, and playback views.
pub(super) struct AppState {
    pub(super) language: Signal<Language>,
    pub(super) apple_theme: Signal<bool>,
    pub(super) archive_path: Signal<String>,
    pub(super) archive_info: Signal<Option<ArchiveInfo>>,
    pub(super) selected_track: Signal<Option<usize>>,
    pub(super) combine_ids: Signal<Vec<usize>>,
    pub(super) playback_selection: Signal<Vec<usize>>,
    pub(super) audio_src: Signal<Option<String>>,
    pub(super) audio_nonce: Signal<usize>,
    pub(super) combined_sources: Signal<Vec<MixedSource>>,
    pub(super) is_paused: Signal<bool>,
    pub(super) current_time: Signal<f64>,
    pub(super) total_time: Signal<f64>,
    pub(super) playback_mode: Signal<String>,
    pub(super) status: Signal<String>,
    pub(super) status_error: Signal<bool>,
    pub(super) editing_track: Signal<bool>,
    pub(super) edit_title: Signal<String>,
    pub(super) edit_artist: Signal<String>,
    pub(super) edit_album: Signal<String>,
    pub(super) edit_genre: Signal<String>,
    pub(super) edit_year: Signal<String>,
    pub(super) edit_cover: Signal<String>,
    pub(super) edit_weight: Signal<String>,
}

/// Create all persistent UI state in a stable hook order.
pub(super) fn use_app_state() -> AppState {
    let language = use_signal(|| Language::TraditionalChinese);
    let apple_theme = use_signal(|| true);
    let archive_path = use_signal(String::new);
    let archive_info = use_signal(|| None::<ArchiveInfo>);
    let selected_track = use_signal(|| None::<usize>);
    let combine_ids = use_signal(Vec::<usize>::new);
    let playback_selection = use_signal(Vec::<usize>::new);
    let audio_src = use_signal(|| None::<String>);
    let audio_nonce = use_signal(|| 0usize);
    let combined_sources = use_signal(Vec::<MixedSource>::new);
    let is_paused = use_signal(|| false);
    let current_time = use_signal(|| 0.0f64);
    let total_time = use_signal(|| 0.0f64);
    let playback_mode = use_signal(|| "idle".to_string());
    let status = use_signal(|| {
        tr(
            Language::TraditionalChinese,
            "載入一個 .mavc 封存即可開始聆聽。",
        )
        .to_string()
    });
    let status_error = use_signal(|| false);
    let editing_track = use_signal(|| false);
    let edit_title = use_signal(String::new);
    let edit_artist = use_signal(String::new);
    let edit_album = use_signal(String::new);
    let edit_genre = use_signal(String::new);
    let edit_year = use_signal(String::new);
    let edit_cover = use_signal(String::new);
    let edit_weight = use_signal(String::new);

    AppState {
        language,
        apple_theme,
        archive_path,
        archive_info,
        selected_track,
        combine_ids,
        playback_selection,
        audio_src,
        audio_nonce,
        combined_sources,
        is_paused,
        current_time,
        total_time,
        playback_mode,
        status,
        status_error,
        editing_track,
        edit_title,
        edit_artist,
        edit_album,
        edit_genre,
        edit_year,
        edit_cover,
        edit_weight,
    }
}

/// Values computed from reactive state for rendering the current screen.
pub(super) struct AppViewModel {
    pub(super) lang: Language,
    pub(super) info: Option<ArchiveInfo>,
    pub(super) selected_id: Option<usize>,
    pub(super) selected: Option<Track>,
    pub(super) current_title: String,
    pub(super) current_artist: String,
    pub(super) detail_artist: String,
    pub(super) detail_album: String,
    pub(super) detail_genre_year: String,
    pub(super) detail_scrape: String,
    pub(super) selected_for_edit: Option<Track>,
    pub(super) selected_track_id: usize,
    pub(super) played_seconds: f64,
    pub(super) total_seconds: f64,
    pub(super) progress_percent: f64,
    pub(super) playback_button_title: &'static str,
    pub(super) playback_button_icon: &'static str,
}

/// Build the display model from the latest archive and player state.
pub(super) fn derive_view_model(state: &AppState) -> AppViewModel {
    let lang = *state.language.read();
    let info = state.archive_info.read().clone();
    let selected_id = *state.selected_track.read();
    let selected = info
        .as_ref()
        .and_then(|archive| {
            archive
                .tracks
                .iter()
                .find(|track| Some(track.id) == selected_id)
        })
        .cloned();
    let current_title = selected
        .as_ref()
        .map(display_title)
        .unwrap_or(tr(lang, "尚未選擇歌曲"))
        .to_owned();
    let current_artist = selected
        .as_ref()
        .and_then(|track| track.metadata.artist.clone())
        .unwrap_or_else(|| {
            info.as_ref()
                .and_then(|archive| archive.creator.clone())
                .unwrap_or_else(|| tr(lang, "載入封存後顯示歌曲資訊").into())
        });
    let detail_artist = selected
        .as_ref()
        .and_then(|track| track.metadata.artist.clone())
        .unwrap_or_else(|| "未標記".into());
    let detail_album = selected
        .as_ref()
        .and_then(|track| track.metadata.album.clone())
        .unwrap_or_else(|| "—".into());
    let detail_genre_year = selected
        .as_ref()
        .map(|track| {
            format!(
                "{} · {}",
                track.metadata.genre.as_deref().unwrap_or("—"),
                track.metadata.year.as_deref().unwrap_or("—")
            )
        })
        .unwrap_or_default();
    let detail_scrape = selected
        .as_ref()
        .map(|track| {
            format!(
                "{} · {} · {} ch · {} bit",
                track
                    .metadata
                    .bitrate_kbps
                    .map(|value| format!("{value} kbps"))
                    .unwrap_or_else(|| "—".into()),
                track
                    .metadata
                    .sample_rate_hz
                    .map(|value| format!("{value} Hz"))
                    .unwrap_or_else(|| "—".into()),
                track
                    .metadata
                    .channels
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".into()),
                track
                    .metadata
                    .bit_depth
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".into())
            )
        })
        .unwrap_or_default();
    let selected_track_id = selected.as_ref().map(|track| track.id).unwrap_or_default();
    let played_seconds = *state.current_time.read();
    let total_seconds = *state.total_time.read();
    let progress_percent = if total_seconds.is_finite() && total_seconds > 0.0 {
        (played_seconds / total_seconds * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let playback_active =
        state.audio_src.read().is_some() || !state.combined_sources.read().is_empty();
    let playback_button_title = if playback_active {
        tr(
            lang,
            if *state.is_paused.read() {
                "繼續播放"
            } else {
                "暫停播放"
            },
        )
    } else {
        tr(lang, "隨機播放 / 合奏")
    };
    let playback_button_icon = if playback_active && !*state.is_paused.read() {
        "Ⅱ"
    } else {
        "▶"
    };

    AppViewModel {
        lang,
        info,
        selected_id,
        selected_for_edit: selected.clone(),
        selected,
        current_title,
        current_artist,
        detail_artist,
        detail_album,
        detail_genre_year,
        detail_scrape,
        selected_track_id,
        played_seconds,
        total_seconds,
        progress_percent,
        playback_button_title,
        playback_button_icon,
    }
}
