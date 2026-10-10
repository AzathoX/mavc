//! Read the desktop language; the UI maps unsupported locales to English.
fn detect() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "-g", "AppleLanguages"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        text.lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && *line != "(" && *line != ")")
            .map(|line| line.trim_matches(['"', ',', ' ']).to_owned())
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[System.Globalization.CultureInfo]::CurrentUICulture.Name",
            ])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8(output.stdout).ok()?.trim().to_owned())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        ["LC_ALL", "LANGUAGE", "LC_MESSAGES", "LANG"]
            .iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.trim().is_empty())
            .map(|value| value.split(':').next().unwrap_or("en").to_owned())
    }
}

#[tauri::command]
pub async fn system_language() -> String {
    tauri::async_runtime::spawn_blocking(|| {
        detect()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "en".into())
    })
    .await
    .unwrap_or_else(|_| "en".into())
}
