mod commands;
mod locale;
mod open_files;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().manage(open_files::OpenFiles::default());
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
        open_files::from_args(app, args, std::path::Path::new(&cwd));
    }));
    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let cwd = std::env::current_dir()?;
            open_files::from_args(app.handle(), std::env::args().collect(), &cwd);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_files::pending_open_files,
            commands::app_version,
            commands::app_info,
            locale::system_language,
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
        .build(tauri::generate_context!())
        .expect("error while building malbum")
        .run(|app, event| {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if let tauri::RunEvent::Opened { urls } = event {
                open_files::enqueue(
                    app,
                    urls.into_iter().filter_map(|url| url.to_file_path().ok()),
                );
            }
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            let _ = (app, event);
        });
}
