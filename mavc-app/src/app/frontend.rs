#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use super::logic::{
    AppState, AppViewModel, ArchiveInfo, Language, MixedSource, PlaybackData, command,
    derive_view_model, display_title, format_time_seconds, optional, seek_audio_to,
    seek_combined_track, set_audio_playing, tr,
};

static CSS: Asset = asset!("/assets/styles.css");

/// Compose the archive workspace, player, track details, and command tools.
pub fn App() -> Element {
    // Create shared state and derive display-only values outside the markup.
    let app_state = super::logic::use_app_state();
    let AppViewModel {
        lang,
        info,
        selected_id,
        selected,
        current_title,
        current_artist,
        detail_artist,
        detail_album,
        detail_genre_year,
        detail_scrape,
        selected_for_edit,
        selected_track_id,
        played_seconds,
        total_seconds,
        progress_percent,
        playback_button_title,
        playback_button_icon,
    } = derive_view_model(&app_state);
    let AppState {
        mut language,
        mut apple_theme,
        mut archive_path,
        mut archive_info,
        mut selected_track,
        mut combine_ids,
        mut playback_selection,
        mut audio_src,
        mut audio_nonce,
        mut combined_sources,
        mut is_paused,
        mut current_time,
        mut total_time,
        mut playback_mode,
        mut status,
        mut status_error,
        mut editing_track,
        mut edit_title,
        mut edit_artist,
        mut edit_album,
        mut edit_genre,
        mut edit_year,
        mut edit_cover,
        mut edit_weight,
    } = app_state;

    rsx! {
        link { rel: "stylesheet", href: CSS }
        main { class: if *apple_theme.read() { "app-shell theme-apple" } else { "app-shell theme-classic" },
            aside { class: "sidebar",
                div { class: "brand-lockup",
                    div { class: "brand-mark", "♫" }
                    div { p { class: "brand-kicker", "MAVC · MUSIC ARCHIVE" } h1 { {tr(lang, "專輯館")} } }
                }
                p { class: "nav-label", {tr(lang, "工作區")} }
                nav { class: "main-nav",
                    a { class: "nav-item active", href: "#player", span { class: "nav-icon", "◉" } span { {tr(lang, "正在聆聽")} } }
                    a { class: "nav-item", href: "#library", span { class: "nav-icon", "▤" } span { {tr(lang, "專輯收藏")} } }
                }
                if let Some(archive) = &info {
                    section { class: "sidebar-track-list", id: "library",
                        div { class: "sidebar-track-heading",
                            p { class: "eyebrow", {tr(lang, "曲目清單 · LIST")} }
                            span { "{archive.tracks.len()}" }
                        }
                        p { class: "sidebar-archive-title", "{archive.title}" }
                        div { class: "sidebar-track-rows",
                            for track in archive.tracks.iter() {
                                {
                                    let track_id = track.id;
                                    let row_class = if Some(track_id) == selected_id { "track-row selected sidebar-track-row" } else { "track-row sidebar-track-row" };
                                    let title = display_title(track).to_owned();
                                    let artist = track.metadata.artist.as_deref().unwrap_or(&track.file_name).to_owned();
                                    rsx! {
                                        div { class: "sidebar-library-row",
                                            button { class: "combine-check sidebar-combine-check", title: tr(lang, "選取作為合奏曲目"), onclick: move |_| {
                                                let mut ids = combine_ids.read().clone();
                                                if ids.contains(&track_id) {
                                                    ids.retain(|id| *id != track_id);
                                                    combine_ids.set(ids);
                                                } else if ids.len() >= 5 {
                                                    status.set(tr(lang, "合奏不能多於五首歌曲。").into());
                                                    status_error.set(true);
                                                } else {
                                                    ids.push(track_id);
                                                    combine_ids.set(ids);
                                                }
                                            }, if combine_ids.read().contains(&track_id) { "☑" } else { "□" } }
                                            button { class: row_class, onclick: move |_| { selected_track.set(Some(track_id)); editing_track.set(false); },
                                                span { class: "track-index", "{track_id:02}" }
                                                span { class: "track-copy", strong { "{title}" } span { "{artist}" } }
                                            }
                                        }
                                    }
                                }
                            }
                            if archive.tracks.is_empty() { p { class: "empty-list", {tr(lang, "這個封存目前沒有曲目。")} } }
                        }
                        if !combine_ids.read().is_empty() {
                            button { class: "sidebar-clear-selection", onclick: move |_| { combined_sources.set(Vec::new()); combine_ids.set(Vec::new()); }, {tr(lang, "清除選取")} }
                        }
                        button { class: "load-track-button sidebar-load-track-button", onclick: move |_| {
                            let path = archive_path.read().clone();
                            let mut status = status; let mut status_error = status_error; let mut archive_info = archive_info;
                            spawn(async move {
                                match command::<Option<String>>("archive_pick_audio", serde_json::json!({})).await {
                                    Ok(Some(file)) => match command::<String>("archive_add", serde_json::json!({"archives": [path.clone()], "files": [file]})).await {
                                        Ok(message) => match command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await {
                                            Ok(updated) => { archive_info.set(Some(updated)); status.set(message); status_error.set(false); }
                                            Err(error) => { status.set(error); status_error.set(true); }
                                        },
                                        Err(error) => { status.set(error); status_error.set(true); }
                                    },
                                    Ok(None) => {}, Err(error) => { status.set(error); status_error.set(true); }
                                }
                            });
                        }, "＋　", {tr(lang, "載入新曲目")} }
                    }
                }
                div { class: "sidebar-spacer" }
                section { class: "collection-card",
                    div { class: "collection-orbit", "𝄞" }
                    p { class: "collection-eyebrow", "MAVC ARCHIVE" }
                    h2 { if let Some(archive) = &info { "{archive.title}" } else { {tr(lang, "專輯館")} } }
                    p { class: "collection-caption", if let Some(archive) = &info { "{archive.tracks.len()} 首曲目已載入" } else { {tr(lang, "加權音樂封存")} } }
                    {
                        let loaded_path = info.as_ref().map(|archive| archive.path.clone()).unwrap_or_default();
                        rsx! { button { class: "text-link", onclick: move |_| {
                        let current_path = loaded_path.clone();
                        let mut archive_path = archive_path;
                        let mut archive_info = archive_info;
                        let mut selected_track = selected_track;
                        let mut combine_ids = combine_ids;
                        let mut status = status;
                        let mut status_error = status_error;
                        status.set(tr(lang, "正在重新載入封存…").into());
                        status_error.set(false);
                        spawn(async move {
                            let path = if current_path.is_empty() {
                                match command::<Option<String>>("archive_pick_existing", serde_json::json!({})).await {
                                    Ok(Some(path)) => path,
                                    Ok(None) => return,
                                    Err(error) => { status.set(error); status_error.set(true); return; }
                                }
                            } else { current_path };
                            match command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await {
                                Ok(archive) => {
                                    archive_path.set(archive.path.clone());
                                    archive_info.set(Some(archive));
                                    selected_track.set(None);
                                    combine_ids.set(Vec::new());
                                    status.set(tr(lang, "封存已重新載入").into());
                                    status_error.set(false);
                                }
                                Err(error) => { status.set(error); status_error.set(true); }
                            }
                        });
                    }, {tr(lang, "重新載入")} } }
                    }
                    button { class: "text-link exit-link", onclick: move |_| {
                        archive_info.set(None);
                        archive_path.set(String::new());
                        selected_track.set(None);
                        combine_ids.set(Vec::new());
                        playback_selection.set(Vec::new());
                        audio_src.set(None);
                        combined_sources.set(Vec::new());
                        is_paused.set(false);
                        current_time.set(0.0);
                        total_time.set(0.0);
                        playback_mode.set("idle".into());
                        editing_track.set(false);
                        edit_title.set(String::new());
                        edit_artist.set(String::new());
                        edit_album.set(String::new());
                        edit_genre.set(String::new());
                        edit_year.set(String::new());
                        edit_cover.set(String::new());
                        edit_weight.set(String::new());
                        audio_nonce += 1;
                        status.set(tr(lang, "載入一個 .mavc 封存即可開始聆聽。").into());
                        status_error.set(false);
                    }, {tr(lang, "退出，回到主界面")} }
                }
                p { class: "sidebar-footnote", "MAVC PLAYER　·　v0.1" }
            }

            // Main workspace: transport controls, archive actions, and track metadata.
            section { class: "workspace",
                header { class: "topbar",
                    div { class: "breadcrumb", span { {tr(lang, "專輯館")} } span { class: "crumb-divider", "/" } strong { if let Some(archive) = &info { "{archive.title}" } else { {tr(lang, "尚未載入封存")} } } }
                    div { class: "topbar-actions",
                        span { class: "today-label", "WEIGHTED MUSIC ARCHIVE" }
                        label { class: "theme-picker", span { {tr(lang, "外觀")} }
                            select {
                                class: "language-select theme-select",
                                value: if *apple_theme.read() { "apple" } else { "classic" },
                                onchange: move |event| apple_theme.set(event.value() == "apple"),
                                option { value: "classic", {tr(lang, "經典真夜")} }
                                option { value: "apple", "Apple" }
                            }
                        }
                        select {
                            class: "language-select",
                            value: "{lang.code()}",
                            onchange: move |event| language.set(Language::from_code(&event.value())),
                            option { value: "zh-CN", "简体中文" }
                            option { value: "zh-TW", "繁體中文" }
                            option { value: "ja", "日本語" }
                            option { value: "en", "English" }
                        }
                    }
                }

                if !status.read().is_empty() {
                    div { class: if *status_error.read() { "status-banner error" } else { "status-banner" }, span { "●" } "{status}" }
                }

                div { class: "content-grid", id: "player",
                    section { class: "player-column",
                        if info.is_some() {
                        div { class: "section-heading",
                            div { p { class: "eyebrow", "MAVC PLAYER" } h2 { {tr(lang, "讓旋律慢一點")} } }
                            button { class: "more-button", onclick: move |_| { archive_info.set(None); audio_src.set(None); }, "↻" }
                        }

                        div { class: "watch-stage",
                            div { class: "watch-chain" }
                            div { class: "watch-bail" }
                            div { class: "watch-case",
                                div { class: "watch-rim",
                                    div { class: "watch-face",
                                        for mark in 0..60 {
                                            span { class: if mark % 5 == 0 { "tick major" } else { "tick" }, style: "--turn: {mark * 6}deg" }
                                        }
                                        span { class: "dial-number n12", "12" }
                                        span { class: "dial-number n3", "3" }
                                        span { class: "dial-number n6", "6" }
                                        span { class: "dial-number n9", "9" }
                                        div { class: "dial-center-label", span { "MUSIC" } strong { "ROOM" } }
                                        div { class: "note-hand",
                                            svg { view_box: "0 0 72 136",
                                                rect { class: "staff-paper", x: "17", y: "18", width: "38", height: "100", rx: "5" }
                                                line { class: "staff-line", x1: "20", y1: "48", x2: "52", y2: "48" }
                                                line { class: "staff-line", x1: "20", y1: "55", x2: "52", y2: "55" }
                                                line { class: "staff-line", x1: "20", y1: "62", x2: "52", y2: "62" }
                                                line { class: "staff-line", x1: "20", y1: "69", x2: "52", y2: "69" }
                                                line { class: "staff-line", x1: "20", y1: "76", x2: "52", y2: "76" }
                                                text { class: "treble-clef", x: "36", y: "111", "𝄞" }
                                            }
                                        }
                                        div { class: "center-pin" }
                                    }
                                }
                            }
                            div { class: "watch-crown" }
                            div { class: "watch-glint" }
                        }

                        div { class: "track-summary",
                            p { class: "eyebrow", "NOW PLAYING" }
                            h3 { "{current_title}" }
                            p { class: "artist-name", "{current_artist}" }
                        }
                        if combined_sources.read().is_empty() {
                            div { class: "music-progress",
                                div { class: "progress-labels",
                                    span { "{format_time_seconds(played_seconds)}" }
                                    span { "{format_time_seconds(total_seconds)}" }
                                }
                                input {
                                    class: "progress-slider",
                                    r#type: "range",
                                    min: "0",
                                    max: "100",
                                    step: "0.1",
                                    value: "{progress_percent}",
                                    style: "--progress: {progress_percent}%",
                                    aria_label: "播放進度",
                                    oninput: move |event| {
                                        if let Ok(percent) = event.value().parse::<f64>() {
                                            let target = total_time.read().max(0.0) * percent.clamp(0.0, 100.0) / 100.0;
                                            seek_audio_to("mavc-main-audio", target);
                                            current_time.set(target);
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "playback-panel",
                            if let Some(source) = audio_src.read().as_ref() {
                                audio {
                                    key: "{audio_nonce}",
                                    class: "audio-player",
                                    id: "mavc-main-audio",
                                    src: "{source}",
                                    controls: false,
                                    autoplay: true,
                                    ontimeupdate: move |event| {
                                        if let Some(dom_event) = event.data.downcast::<web_sys::Event>() {
                                            if let Some(audio) = dom_event.current_target().and_then(|target| target.dyn_into::<web_sys::HtmlAudioElement>().ok()) {
                                                let position = audio.current_time();
                                                if (position - *current_time.read()).abs() >= 0.1 {
                                                    current_time.set(position);
                                                }
                                            }
                                        }
                                    },
                                    onloadedmetadata: move |event| {
                                        if let Some(dom_event) = event.data.downcast::<web_sys::Event>() {
                                            if let Some(audio) = dom_event.current_target().and_then(|target| target.dyn_into::<web_sys::HtmlAudioElement>().ok()) {
                                                current_time.set(audio.current_time());
                                                total_time.set(audio.duration());
                                            }
                                        }
                                    },
                                    onerror: move |_| {
                                        status.set(tr(lang, "目前的播放器無法解碼這種音訊格式。").into());
                                        status_error.set(true);
                                    },
                                    onplay: move |_| is_paused.set(false),
                                    onpause: move |_| is_paused.set(true),
                                    onended: move |_| {
                                        if *playback_mode.read() == "all" {
                                            let path = archive_path.read().clone();
                                            let ids = archive_info.read().as_ref().map(|archive| archive.tracks.iter().map(|track| track.id).collect::<Vec<_>>()).unwrap_or_default();
                                            let current = *selected_track.read();
                                            if let Some(next) = current.and_then(|id| ids.iter().position(|item| *item == id)).and_then(|index| ids.get(index + 1).copied()) {
                                                selected_track.set(Some(next));
                                    let mut audio_src = audio_src;
                                    let mut audio_nonce = audio_nonce;
                                                let mut status = status;
                                                spawn(async move {
                                                    match command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": next})).await {
                                                        Ok(playback) => { current_time.set(0.0); total_time.set(playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0)); audio_src.set(Some(playback.src.clone())); audio_nonce += 1; status.set(format!("正在播放《{}》：{}", playback.archive_title, display_title(&playback.track))); }
                                                        Err(error) => { status.set(error); }
                                                    }
                                                });
                                            } else {
                                                playback_mode.set("idle".into());
                                                audio_src.set(None);
                                                is_paused.set(false);
                                                status.set(tr(lang, "播放清單已播放完畢。").into());
                                            }
                                        } else {
                                            audio_src.set(None);
                                            is_paused.set(false);
                                        }
                                    }
                                }
                            } else {
                                div { class: "audio-placeholder", {tr(lang, "載入封存並選擇歌曲後，播放器會顯示在這裡。")} }
                            }
                            div { class: "transport-controls command-transport",
                                button { class: "transport-button secondary", title: tr(lang, "隨機播放"), onclick: move |_| {
                                    let path = archive_path.read().clone();
                                    let mut audio_src = audio_src; let mut audio_nonce = audio_nonce; let mut status = status; let mut status_error = status_error;
                                    combined_sources.set(Vec::new());
                                    playback_selection.set(Vec::new());
                                    is_paused.set(false);
                                    playback_mode.set("single".into());
                                    spawn(async move { match command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": null})).await {
                                        Ok(playback) => { selected_track.set(Some(playback.track.id)); current_time.set(0.0); total_time.set(playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0)); audio_src.set(Some(playback.src.clone())); audio_nonce += 1; status.set(format!("隨機播放《{}》：{}", playback.archive_title, display_title(&playback.track))); status_error.set(false); }
                                        Err(error) => { status.set(error); status_error.set(true); }
                                    }});
                                }, "⤨" }
                                button { class: "play-button", title: "{playback_button_title}", onclick: move |_| {
                                    let checked_ids = combine_ids.read().clone();
                                    if checked_ids.len() > 5 {
                                        status.set(tr(lang, "合奏不能多於五首歌曲。").into());
                                        status_error.set(true);
                                        return;
                                    }
                                    let active_sources = combined_sources.read().clone();
                                    let has_single_source = audio_src.read().is_some();
                                    if has_single_source || !active_sources.is_empty() {
                                        if checked_ids == *playback_selection.read() {
                                            let resume = *is_paused.read();
                                            if active_sources.is_empty() {
                                                set_audio_playing("mavc-main-audio", resume);
                                            } else {
                                                for source in &active_sources {
                                                    set_audio_playing(&format!("mavc-combined-audio-{}", source.id), resume);
                                                }
                                            }
                                            is_paused.set(!resume);
                                            return;
                                        }
                                        if has_single_source {
                                            set_audio_playing("mavc-main-audio", false);
                                            audio_src.set(None);
                                        } else {
                                            for source in &active_sources {
                                                set_audio_playing(&format!("mavc-combined-audio-{}", source.id), false);
                                            }
                                            combined_sources.set(Vec::new());
                                        }
                                    }
                                    playback_selection.set(checked_ids.clone());
                                    is_paused.set(false);
                                    let path = archive_path.read().clone();
                                    let mut audio_src = audio_src;
                                    let mut audio_nonce = audio_nonce;
                                    let mut status = status;
                                    let mut status_error = status_error;
                                    if checked_ids.len() == 1 {
                                        let id = checked_ids[0];
                                        combined_sources.set(Vec::new());
                                        is_paused.set(false);
                                        playback_mode.set("single".into());
                                        spawn(async move {
                                            match command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": id})).await {
                                                Ok(playback) => {
                                                    selected_track.set(Some(playback.track.id));
                                                    current_time.set(0.0);
                                                    total_time.set(playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0));
                                                    audio_src.set(Some(playback.src.clone()));
                                                    audio_nonce += 1;
                                                    status.set(format!("正在播放勾選曲目《{}》", display_title(&playback.track)));
                                                    status_error.set(false);
                                                }
                                                Err(error) => { status.set(error); status_error.set(true); }
                                            }
                                        });
                                        return;
                                    }
                                    if checked_ids.len() >= 2 {
                                        is_paused.set(false);
                                        playback_mode.set("idle".into());
                                        spawn(async move {
                                            match command::<Vec<PlaybackData>>("archive_play_combine", serde_json::json!({"path": path, "trackIds": checked_ids})).await {
                                                Ok(playbacks) => {
                                                    current_time.set(0.0);
                                                    total_time.set(0.0);
                                                    if let Some(first) = playbacks.first() {
                                                        selected_track.set(Some(first.track.id));
                                                    }
                                                    let sources = playbacks.into_iter().map(|playback| MixedSource {
                                                        id: playback.track.id,
                                                        title: display_title(&playback.track).to_owned(),
                                                        src: playback.src.clone(),
                                                        position_seconds: 0.0,
                                                        duration_seconds: playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0),
                                                    }).collect();
                                                    audio_src.set(None);
                                                    audio_nonce += 1;
                                                    combined_sources.set(sources);
                                                    status.set(tr(lang, "正在同步播放所選曲目。").into());
                                                    status_error.set(false);
                                                }
                                                Err(error) => { status.set(error); status_error.set(true); }
                                            }
                                        });
                                    } else {
                                        let id = None::<usize>;
                                        combined_sources.set(Vec::new());
                                        is_paused.set(false);
                                        playback_mode.set("single".into());
                                        spawn(async move { match command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": id})).await {
                                            Ok(playback) => {
                                                selected_track.set(Some(playback.track.id));
                                                current_time.set(0.0);
                                                total_time.set(playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0));
                                                audio_src.set(Some(playback.src.clone()));
                                                audio_nonce += 1;
                                                status.set(format!("隨機播放《{}》：{}", playback.archive_title, display_title(&playback.track)));
                                                status_error.set(false);
                                            }
                                            Err(error) => { status.set(error); status_error.set(true); }
                                        }});
                                    }
                                }, " {playback_button_icon}" }
                                button { class: "transport-button secondary", title: tr(lang, "依序播放全部"), onclick: move |_| {
                                    let path = archive_path.read().clone();
                                    let first_id = archive_info.read().as_ref().and_then(|archive| archive.tracks.first().map(|track| track.id));
                                    let mut audio_src = audio_src; let mut audio_nonce = audio_nonce; let mut status = status; let mut status_error = status_error;
                                    if let Some(id) = first_id {
                                        combined_sources.set(Vec::new());
                                        playback_selection.set(Vec::new());
                                        is_paused.set(false); selected_track.set(Some(id)); playback_mode.set("all".into());
                                        spawn(async move { match command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": id})).await {
                                            Ok(playback) => { current_time.set(0.0); total_time.set(playback.track.metadata.duration_ms.map(|duration| duration as f64 / 1000.0).unwrap_or(0.0)); audio_src.set(Some(playback.src.clone())); audio_nonce += 1; status.set(format!("依序播放《{}》（從 {} 開始）", playback.archive_title, display_title(&playback.track))); status_error.set(false); }
                                            Err(error) => { status.set(error); status_error.set(true); }
                                        }});
                                    } else { status.set(tr(lang, "請先載入一個封存。").into()); status_error.set(true); }
                                }, "▶▶" }
                                button { class: "transport-button queue-toggle", title: tr(lang, "系統播放器播放清單"), onclick: move |_| {
                                    let path = archive_path.read().clone(); let mut status = status; let mut status_error = status_error;
                                    audio_src.set(None); combined_sources.set(Vec::new()); playback_mode.set("idle".into());
                                    spawn(async move { match command::<String>("archive_system_playlist", serde_json::json!({"path": path})).await {
                                        Ok(message) => { status.set(message); status_error.set(false); }
                                        Err(error) => { status.set(error); status_error.set(true); }
                                    }});
                                }, "⇧" }
                            }
                            if !combined_sources.read().is_empty() {
                                div { class: "combine-audio-list",
                                    for source in combined_sources.read().iter() {
                                        {
                                            let source_id = source.id;
                                            let duration_seconds = source.duration_seconds;
                                            let percent = if source.duration_seconds > 0.0 {
                                                (source.position_seconds / source.duration_seconds * 100.0).clamp(0.0, 100.0)
                                            } else { 0.0 };
                                            rsx! {
                                                div { class: "combine-audio-row",
                                                    div { class: "combine-progress-heading",
                                                        strong { "{source.id:02} · {source.title}" }
                                                        span { "{format_time_seconds(source.position_seconds)} / {format_time_seconds(source.duration_seconds)}" }
                                                    }
                                                    input {
                                                        class: "progress-slider combine-progress-slider",
                                                        r#type: "range",
                                                        min: "0",
                                                        max: "100",
                                                        step: "0.1",
                                                        value: "{percent}",
                                                        style: "--progress: {percent}%",
                                                        aria_label: "曲目播放進度",
                                                        oninput: move |event| {
                                                            if let Ok(percent) = event.value().parse::<f64>() {
                                                                seek_audio_to(&format!("mavc-combined-audio-{source_id}"), duration_seconds * percent.clamp(0.0, 100.0) / 100.0);
                                                            }
                                                        }
                                                    }
                                                    div { class: "combine-seek-controls",
                                                        button { class: "seek-step-button", title: "後退 0.5 秒", aria_label: "後退 0.5 秒", onclick: move |_| seek_combined_track(source_id, -0.5), "−0.5 s" }
                                                        button { class: "seek-step-button", title: "前進 0.5 秒", aria_label: "前進 0.5 秒", onclick: move |_| seek_combined_track(source_id, 0.5), "+0.5 s" }
                                                    }
                                                    audio {
                                                        id: "mavc-combined-audio-{source_id}", class: "combined-audio-stream", src: "{source.src}", controls: false, autoplay: true,
                                                        ontimeupdate: move |event| {
                                                            if let Some(dom_event) = event.data.downcast::<web_sys::Event>() {
                                                                if let Some(audio) = dom_event.current_target().and_then(|target| target.dyn_into::<web_sys::HtmlAudioElement>().ok()) {
                                                                    let position = audio.current_time();
                                                                    let mut sources = combined_sources.read().clone();
                                                                    if let Some(source) = sources.iter_mut().find(|source| source.id == source_id) {
                                                                        source.position_seconds = position;
                                                                        combined_sources.set(sources);
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        onloadedmetadata: move |event| {
                                                            if let Some(dom_event) = event.data.downcast::<web_sys::Event>() {
                                                                if let Some(audio) = dom_event.current_target().and_then(|target| target.dyn_into::<web_sys::HtmlAudioElement>().ok()) {
                                                                    let mut sources = combined_sources.read().clone();
                                                                    if let Some(source) = sources.iter_mut().find(|source| source.id == source_id) {
                                                                        source.duration_seconds = audio.duration();
                                                                        combined_sources.set(sources);
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        onerror: move |_| {
                                                            status.set(tr(lang, "目前的播放器無法解碼這種音訊格式。").into());
                                                            status_error.set(true);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        } else {
                            div { class: "empty-state",
                                div { class: "empty-state-actions",
                                    button { class: "make-archive-button", onclick: move |_| {
                                        let mut status = status;
                                        let mut status_error = status_error;
                                        let mut archive_info = archive_info;
                                        let mut archive_path = archive_path;
                                        let mut selected_track = selected_track;
                                        spawn(async move {
                                            match command::<Option<ArchiveInfo>>("archive_create_music_from_picker", serde_json::json!({})).await {
                                                Ok(Some(archive)) => {
                                                    archive_path.set(archive.path.clone());
                                                    selected_track.set(None);
                                                    status.set(format!("{}{}", tr(lang, "已建立 MAVC 封存："), archive.path));
                                                    status_error.set(false);
                                                    archive_info.set(Some(archive));
                                                }
                                                Ok(None) => {}
                                                Err(error) => {
                                                    status.set(if error == "Please select at least two audio files." {
                                                        tr(lang, "選擇至少兩個音訊檔").to_owned()
                                                    } else { error });
                                                    status_error.set(true);
                                                }
                                            }
                                        });
                                    },
                                        span { {tr(lang, "製作音樂館")} }
                                        small { {tr(lang, "選擇至少兩個音訊檔")} }
                                    }
                                    button { class: "import-archive-button", onclick: move |_| {
                                        let mut status = status;
                                        let mut status_error = status_error;
                                        let mut archive_info = archive_info;
                                        let mut archive_path = archive_path;
                                        let mut selected_track = selected_track;
                                        spawn(async move {
                                            match command::<Option<String>>("archive_pick_existing", serde_json::json!({})).await {
                                                Ok(Some(path)) => match command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await {
                                                    Ok(archive) => {
                                                        archive_path.set(archive.path.clone());
                                                        selected_track.set(None);
                                                        status.set(format!("{}{}", tr(lang, "已載入 MAVC 封存："), archive.title));
                                                        status_error.set(false);
                                                        archive_info.set(Some(archive));
                                                    }
                                                    Err(error) => { status.set(error); status_error.set(true); }
                                                },
                                                Ok(None) => {}
                                                Err(error) => { status.set(error); status_error.set(true); }
                                            }
                                        });
                                    },
                                        span { {tr(lang, "導入音樂館")} }
                                        small { "EDIT MAVC" }
                                    }
                                }
                            }
                        }
                    }

                    aside { class: "details-column",
                        section { class: "archive-card archive-load-card",
                            div { class: "card-topline", span { {tr(lang, "Explore · 導出封存")} } span { class: "live-dot" } }
                            div { class: "path-input-row",
                                input { class: "tool-input", placeholder: "/path/to/archive.mavc", value: "{archive_path}", readonly: true }
                                button { class: "small-gold-button", disabled: info.is_none(), onclick: move |_| {
                                    let source = archive_path.read().trim().to_owned();
                                    let mut status = status;
                                    let mut status_error = status_error;
                                    if source.is_empty() {
                                        status.set(tr(lang, "請先載入 MAVC 封存。").into());
                                        status_error.set(true);
                                    } else {
                                        spawn(async move {
                                            match command::<Option<String>>("archive_export_picker", serde_json::json!({"source": source})).await {
                                                Ok(Some(path)) => {
                                                    status.set(format!("{}{}", tr(lang, "已導出封存："), path));
                                                    status_error.set(false);
                                                }
                                                Ok(None) => {}
                                                Err(error) => { status.set(error); status_error.set(true); }
                                            }
                                        });
                                    }
                                }, {tr(lang, "導出")} }
                            }
                            if let Some(archive) = &info {
                                h3 { "{archive.title}" }
                                p { class: "album-meta", if let Some(creator) = &archive.creator { "{creator} · {archive.tracks.len()} 首" } else { "{archive.tracks.len()} 首曲目" } }
                                if let Some(description) = &archive.description { p { class: "archive-description", "{description}" } }
                            } else {
                                p { class: "album-meta", {tr(lang, "輸入封存檔案路徑以載入曲目。")} }
                            }
                        }

                        if let Some(track) = &selected {
                            section { class: "track-detail-card",
                                if let Some(url) = track.embedded_cover_data_url.as_ref().or(track.cover_art_url.as_ref()) { img { class: "track-cover-image", src: "{url}", alt: "曲目封面" } }
                                div { class: "track-detail-heading",
                                    div { p { class: "eyebrow", "曲目資訊 · TRACK DETAILS" } h3 { "{display_title(track)}" } }
                                    button { class: "edit-track-button", onclick: move |_| {
                                        if let Some(track) = selected_for_edit.clone() {
                                            edit_title.set(display_title(&track).to_owned());
                                            edit_artist.set(track.metadata.artist.clone().unwrap_or_default());
                                            edit_album.set(track.metadata.album.clone().unwrap_or_default());
                                            edit_genre.set(track.metadata.genre.clone().unwrap_or_default());
                                            edit_year.set(track.metadata.year.clone().unwrap_or_default());
                                            edit_cover.set(track.cover_art_url.clone().unwrap_or_default());
                                            edit_weight.set(track.weight.to_string());
                                        }
                                        editing_track.set(true);
                                    }, "編輯" }
                                }
                                div { class: "track-detail-grid",
                                    span { {tr(lang, "歌手")} } strong { "{detail_artist}" }
                                    span { {tr(lang, "權重")} } strong { "{track.weight:.3}" }
                                    span { {tr(lang, "專輯")} } strong { "{detail_album}" }
                                    span { {tr(lang, "曲風 / 年份")} } strong { "{detail_genre_year}" }
                                    span { {tr(lang, "刮削資訊")} } strong { "{detail_scrape}" }
                                }
                                if *editing_track.read() {
                                    div { class: "track-edit-form",
                                        input { class: "tool-input", placeholder: tr(lang, "曲名"), value: "{edit_title}", oninput: move |e| edit_title.set(e.value()) }
                                        input { class: "tool-input", placeholder: "歌手", value: "{edit_artist}", oninput: move |e| edit_artist.set(e.value()) }
                                        input { class: "tool-input", placeholder: "專輯", value: "{edit_album}", oninput: move |e| edit_album.set(e.value()) }
                                        input { class: "tool-input", placeholder: "曲風", value: "{edit_genre}", oninput: move |e| edit_genre.set(e.value()) }
                                        input { class: "tool-input", placeholder: "年份", value: "{edit_year}", oninput: move |e| edit_year.set(e.value()) }
                                        input { class: "tool-input", placeholder: tr(lang, "封面圖片 URL"), value: "{edit_cover}", oninput: move |e| edit_cover.set(e.value()) }
                                        input { class: "tool-input", placeholder: "權重 0–1", value: "{edit_weight}", oninput: move |e| edit_weight.set(e.value()) }
                                        div { class: "track-edit-actions",
                                            button { class: "small-gold-button", onclick: move |_| {
                                                let weight = edit_weight.read().parse::<f64>().ok();
                                                if weight.is_none() { status.set(tr(lang, "權重需介於 0 和 1").into()); status_error.set(true); return; }
                                                let path = archive_path.read().clone();
                                                let title = edit_title.read().clone(); let artist = edit_artist.read().clone(); let album = edit_album.read().clone(); let genre = edit_genre.read().clone(); let year = edit_year.read().clone(); let cover = edit_cover.read().clone();
                                                let mut archive_info = archive_info; let mut status = status; let mut status_error = status_error; let mut editing_track = editing_track;
                                                spawn(async move {
                                                    match command::<()>("archive_update_track_details", serde_json::json!({"path": path, "trackId": selected_track_id, "title": title, "artist": optional(artist), "album": optional(album), "genre": optional(genre), "year": optional(year), "coverArtUrl": optional(cover), "weight": weight.unwrap()})).await {
                                                        Ok(()) => match command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await { Ok(updated) => { archive_info.set(Some(updated)); status.set(tr(lang, "曲目資訊已更新").into()); status_error.set(false); editing_track.set(false); }, Err(error) => { status.set(error); status_error.set(true); } },
                                                        Err(error) => { status.set(error); status_error.set(true); }
                                                    }
                                                });
                                            }, {tr(lang, "儲存")} }
                                            button { class: "cancel-edit-button", onclick: move |_| editing_track.set(false), {tr(lang, "取消")} }
                                        }
                                    }
                                }
                                div { class: "track-detail-actions",
                                    button { class: "remove-track-button", onclick: move |_| {
                                        let path = archive_path.read().clone(); let track_id = selected_track_id;
                                        let mut archive_info = archive_info; let mut selected_track = selected_track; let mut status = status; let mut status_error = status_error;
                                        spawn(async move {
                                            match command::<String>("archive_remove", serde_json::json!({"archives": [path.clone()], "trackIds": [track_id]})).await {
                                                Ok(message) => match command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await { Ok(updated) => { archive_info.set(Some(updated)); selected_track.set(None); status.set(message); status_error.set(false); }, Err(error) => { status.set(error); status_error.set(true); } },
                                                Err(error) => { status.set(error); status_error.set(true); }
                                            }
                                        });
                                    }, {tr(lang, "移除曲目")} }
                                }
                            }
                        }

                    }
                }

                footer { class: "footer-note", span { "◌" } " " {tr(lang, "所有封存操作皆由本機 mavc 函式庫執行")} span { class: "footer-version", "MAVC PLAYER · 0.1" } }
            }
        }
    }
}
