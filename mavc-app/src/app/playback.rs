//! Browser audio element controls used by single-track and combined playback.

use wasm_bindgen::JsCast;

pub(super) fn seek_combined_track(track_id: usize, offset_seconds: f64) {
    let Some(audio) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("mavc-combined-audio-{track_id}")))
        .and_then(|element| element.dyn_into::<web_sys::HtmlAudioElement>().ok())
    else {
        return;
    };
    let mut target = (audio.current_time() + offset_seconds).max(0.0);
    let duration = audio.duration();
    if duration.is_finite() {
        target = target.min(duration);
    }
    let _ = audio.set_current_time(target);
}

pub(super) fn seek_audio_to(element_id: &str, target_seconds: f64) {
    let Some(audio) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(element_id))
        .and_then(|element| element.dyn_into::<web_sys::HtmlAudioElement>().ok())
    else {
        return;
    };
    let mut target = target_seconds.max(0.0);
    let duration = audio.duration();
    if duration.is_finite() {
        target = target.min(duration);
    }
    let _ = audio.set_current_time(target);
}

pub(super) fn set_audio_playing(element_id: &str, playing: bool) {
    let Some(audio) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(element_id))
        .and_then(|element| element.dyn_into::<web_sys::HtmlAudioElement>().ok())
    else {
        return;
    };
    if playing {
        let _ = audio.play();
    } else {
        let _ = audio.pause();
    }
}
