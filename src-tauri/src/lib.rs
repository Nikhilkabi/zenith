mod commands;
mod db;
mod images;
mod models;

use db::init_library;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let state = init_library().expect("Failed to initialize Bucket library");
            app.manage(Mutex::new(state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_countries,
            commands::create_country,
            commands::get_library_path,
            commands::reveal_library,
            commands::list_places,
            commands::get_place,
            commands::create_place,
            commands::list_images,
            commands::import_image_bytes,
            commands::import_image_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
