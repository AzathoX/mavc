//! Typed wrapper around Tauri commands invoked by the webview.

use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
let pending = false;
let wake;
export function desktop_available() { return !!window.__TAURI__?.core; }
export function browser_language() { return navigator.languages?.[0] || navigator.language || 'en'; }
export function set_boot_language(code) { window.malbumSetLanguage?.(code); }
export async function watch_open_files() {
    await window.__TAURI__.event.listen('malbum-open-files-changed', () => {
        if (wake) { const resolve = wake; wake = undefined; resolve(); }
        else { pending = true; }
    });
}
export async function next_open_files_event() {
    if (pending) { pending = false; return; }
    await new Promise(resolve => { wake = resolve; });
}
export async function reveal_interface() { await window.malbumReveal?.(); }
")]
extern "C" {
    pub(super) fn desktop_available() -> bool;
    pub(super) fn browser_language() -> String;
    pub(super) fn set_boot_language(code: &str);
    #[wasm_bindgen(catch)]
    pub(super) async fn watch_open_files() -> Result<(), JsValue>;
    pub(super) async fn next_open_files_event();
    pub(super) async fn reveal_interface();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(command: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub(crate) async fn command<T: DeserializeOwned>(
    name: &str,
    args: serde_json::Value,
) -> Result<T, String> {
    web_sys::console::log_2(&"[MAVC] invoke".into(), &format!("{name} {args}").into());
    let args = serde_wasm_bindgen::to_value(&args).map_err(|error| error.to_string())?;
    let value = match invoke(name, args).await {
        Ok(value) => {
            web_sys::console::log_2(&"[MAVC] success".into(), &name.into());
            value
        }
        Err(error) => {
            let message = error.as_string().unwrap_or_else(|| format!("{error:?}"));
            web_sys::console::error_2(&"[MAVC] error".into(), &format!("{name}: {message}").into());
            return Err(message);
        }
    };
    serde_wasm_bindgen::from_value(value).map_err(|error| {
        web_sys::console::error_2(
            &"[MAVC] decode error".into(),
            &format!("{name}: {error}").into(),
        );
        error.to_string()
    })
}
