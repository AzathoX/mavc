mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::archive_list,
            commands::archive_inspect,
            commands::archive_create_music,
            commands::archive_create_music_from_picker,
            commands::archive_pick_existing,
            commands::archive_pick_audio,
            commands::archive_update_track_details,
            commands::archive_export_picker,
            commands::archive_create_manifest,
            commands::archive_add,
            commands::archive_remove,
            commands::archive_pick,
            commands::archive_weight,
            commands::archive_extract_one,
            commands::archive_extract_all,
            commands::archive_play_track,
            commands::archive_play_combine,
            commands::archive_system_playlist,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
