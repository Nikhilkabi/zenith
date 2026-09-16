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
            commands::update_place,
            commands::delete_place,
            commands::list_images,
            commands::import_image_bytes,
            commands::import_image_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, init_library_at};
    use image::ImageEncoder;
    use uuid::Uuid;

    fn jpeg_bytes() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(48, 32, image::Rgb([210, 72, 54]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 80)
            .write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgb8,
            )
            .expect("encode jpeg");
        bytes
    }

    #[test]
    fn country_place_photo_roundtrip() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).expect("init temp library");

        let country = db::create_country(&state.conn, "Japan", Some("JP")).unwrap();
        let place = db::create_place(&state.conn, &country.id, "Kyoto", Some("been")).unwrap();
        assert_eq!(place.status, "been");

        let rec = images::import_image_bytes(&state, &place.id, "kyoto.jpg", jpeg_bytes())
            .expect("import jpeg");

        let thumb = state.library_root.join(rec.thumb_relpath.replace('/', std::path::MAIN_SEPARATOR_STR));
        let display =
            state.library_root.join(rec.display_relpath.replace('/', std::path::MAIN_SEPARATOR_STR));
        assert!(thumb.exists(), "thumb missing: {thumb:?}");
        assert!(display.exists(), "display missing: {display:?}");

        let listed = db::list_images(&state.conn, &place.id).unwrap();
        assert_eq!(listed.len(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_heic() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Iceland", None).unwrap();
        let place = db::create_place(&state.conn, &country.id, "Reykjavik", None).unwrap();

        let err = images::import_image_bytes(&state, &place.id, "shot.heic", b"not-an-image".to_vec())
            .unwrap_err();
        assert!(err.to_string().contains("HEIC"), "{err}");

        let _ = std::fs::remove_dir_all(dir);
    }
}
