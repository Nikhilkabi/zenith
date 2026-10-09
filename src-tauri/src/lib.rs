mod capture;
mod commands;
mod covers;
mod db;
mod export;
mod images;
mod models;

use db::init_library;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
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
            app.manage(Arc::new(Mutex::new(state)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_countries,
            commands::create_country,
            commands::update_country,
            commands::delete_country,
            commands::get_library_path,
            commands::reveal_library,
            commands::open_url,
            commands::list_places,
            commands::list_all_places,
            commands::get_place,
            commands::create_place,
            commands::update_place,
            commands::delete_place,
            commands::list_images,
            commands::delete_image,
            commands::set_place_cover,
            commands::set_country_cover,
            commands::import_image_bytes,
            commands::import_image_path,
            commands::list_links,
            commands::add_link,
            commands::delete_link,
            commands::search,
            commands::export_library_zip,
            commands::import_library_zip,
            commands::sweep_orphans,
            commands::list_country_photos,
            commands::reorder_countries,
            commands::reorder_places,
            commands::update_image_caption,
            commands::list_trash,
            commands::restore_trash,
            commands::purge_trash,
            commands::drop_on_country,
            commands::list_goals,
            commands::get_goal,
            commands::create_goal,
            commands::update_goal,
            commands::delete_goal,
            commands::reorder_goals,
            commands::list_trips,
            commands::get_trip,
            commands::create_trip,
            commands::update_trip,
            commands::delete_trip,
            commands::add_trip_stop,
            commands::remove_trip_stop,
            commands::reorder_trip_stops,
            commands::set_trip_cost,
            commands::list_place_tasks,
            commands::add_place_task,
            commands::set_place_task,
            commands::delete_place_task,
            commands::cover_credit,
            commands::get_photo_sources,
            commands::set_photo_sources,
            commands::import_stock_photo,
            commands::search_covers,
            commands::apply_stock_cover,
            commands::apply_file_cover,
            commands::set_cover_focus,
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

        db::set_place_cover(&state.conn, &place.id, &rec.id).unwrap();
        db::set_country_cover(&state.conn, &country.id, &rec.id).unwrap();
        let country = db::get_country(&state.conn, &country.id).unwrap();
        assert!(country.cover_relpath.is_some());

        db::delete_image(&state, &rec.id).unwrap();
        assert!(db::list_images(&state.conn, &place.id).unwrap().is_empty());

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

    #[test]
    fn import_stock_rejects_unknown_id() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Indonesia", None).unwrap();
        let place = db::create_place(&state.conn, &country.id, "Bromo", None).unwrap();
        let err = crate::covers::import_stock_photo(&state, &place.id, "not-a-uuid").unwrap_err();
        assert!(err.to_string().contains("Unknown"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn search_place_and_export() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Japan", Some("JP")).unwrap();
        db::create_place(&state.conn, &country.id, "Kyoto", None).unwrap();

        let hits = db::search(&state.conn, "kyo").unwrap();
        assert!(hits.iter().any(|h| h.title == "Kyoto"));

        let zip_path = std::env::temp_dir().join(format!("bucket-export-{}.zip", Uuid::new_v4()));
        crate::export::zip_library(&state.library_root, &zip_path).unwrap();
        assert!(zip_path.exists());
        let _ = std::fs::remove_file(zip_path);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn search_escapes_like_wildcards() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        db::create_country(&state.conn, "Japan", None).unwrap();
        db::create_country(&state.conn, "Iceland", None).unwrap();
        let hits = db::search(&state.conn, "_").unwrap();
        assert!(hits.is_empty(), "underscore should not match every name: {hits:?}");
        let japan = db::search(&state.conn, "Jap").unwrap();
        assert!(japan.iter().any(|h| h.title == "Japan"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn search_includes_links() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Japan", Some("JP")).unwrap();
        db::insert_link(
            &state.conn,
            "https://example.com/ryokan",
            Some("Kyoto ryokan"),
            None,
            "web",
            None,
            Some(&country.id),
        )
        .unwrap();
        let hits = db::search(&state.conn, "ryokan").unwrap();
        assert!(hits.iter().any(|h| h.kind == "link" && h.title.contains("ryokan")));
        let bare = db::insert_link(
            &state.conn,
            "https://www.youtube.com/watch?v=abc",
            None,
            None,
            "youtube",
            None,
            Some(&country.id),
        )
        .unwrap();
        db::set_link_preview(&state.conn, &bare.id, Some("Waterfall guide"), Some("library/link-thumbs/x.jpg")).unwrap();
        let saved = db::list_links(&state.conn, None, Some(&country.id)).unwrap();
        let video = saved.iter().find(|link| link.id == bare.id).unwrap();
        assert_eq!(video.title.as_deref(), Some("Waterfall guide"));
        assert_eq!(video.thumb_relpath.as_deref(), Some("library/link-thumbs/x.jpg"));
        db::set_link_preview(&state.conn, &bare.id, Some("Other name"), Some("library/link-thumbs/y.jpg")).unwrap();
        let again = db::list_links(&state.conn, None, Some(&country.id)).unwrap();
        let video = again.iter().find(|link| link.id == bare.id).unwrap();
        assert_eq!(video.title.as_deref(), Some("Waterfall guide"));
        assert_eq!(video.thumb_relpath.as_deref(), Some("library/link-thumbs/x.jpg"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn goal_cover_roundtrip_and_search() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let goal = db::create_goal(&state.conn, "Run a marathon").unwrap();
        assert_eq!(goal.status, "dream");
        db::update_goal(&state.conn, &goal.id, None, Some("Boston"), Some("done")).unwrap();
        let hits = db::search(&state.conn, "marathon").unwrap();
        assert!(hits.iter().any(|h| h.kind == "goal"));
        let notes = db::search(&state.conn, "Boston").unwrap();
        assert!(notes.iter().any(|h| h.kind == "goal"));

        crate::covers::apply_file(&state, "goal", &goal.id, "cover.jpg", &jpeg_bytes(), 20.0, 80.0)
            .unwrap();
        let goal = db::get_goal(&state.conn, &goal.id).unwrap();
        let rel = goal.cover_relpath.expect("cover");
        assert!(state.library_root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR)).exists());
        assert!(goal.cover_credit.is_none());
        assert!((goal.cover_x - 20.0).abs() < 0.01);
        assert!((goal.cover_y - 80.0).abs() < 0.01);
        db::set_cover_focus(&state.conn, "goal", &goal.id, 140.0, -5.0).unwrap();
        let goal = db::get_goal(&state.conn, &goal.id).unwrap();
        assert!((goal.cover_x - 100.0).abs() < 0.01);
        assert!((goal.cover_y - 0.0).abs() < 0.01);

        db::delete_goal(&state.conn, &goal.id).unwrap();
        assert!(db::list_goals(&state.conn).unwrap().is_empty());
        db::restore_trash(&state.conn, "goal", &goal.id).unwrap();
        assert_eq!(db::list_goals(&state.conn).unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_duplicate_country() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        db::create_country(&state.conn, "Indonesia", Some("ID")).unwrap();
        let err = db::create_country(&state.conn, "indonesia", None).unwrap_err();
        assert!(err.to_string().contains("already"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn photo_dedupe_and_caption() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Japan", None).unwrap();
        let place = db::create_place(&state.conn, &country.id, "Kyoto", None).unwrap();
        let bytes = jpeg_bytes();
        let first = images::import_image_bytes(&state, &place.id, "a.jpg", bytes.clone()).unwrap();
        let second = images::import_image_bytes(&state, &place.id, "b.jpg", bytes).unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(db::list_images(&state.conn, &place.id).unwrap().len(), 1);

        let updated = db::update_image_caption(&state.conn, &first.id, Some("Kiyomizu")).unwrap();
        assert_eq!(updated.caption.as_deref(), Some("Kiyomizu"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn trash_restore_and_country_photos() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Japan", None).unwrap();
        let place = db::create_place(&state.conn, &country.id, "Kyoto", None).unwrap();
        images::import_image_bytes(&state, &place.id, "a.jpg", jpeg_bytes()).unwrap();
        assert_eq!(db::list_country_photos(&state.conn, &country.id, 8).unwrap().len(), 1);

        db::delete_country(&state.conn, &country.id).unwrap();
        assert!(db::list_countries(&state.conn).unwrap().is_empty());
        let trash = db::list_trash(&state.conn).unwrap();
        assert!(trash.iter().any(|t| t.kind == "country" && t.id == country.id));

        db::restore_trash(&state.conn, "country", &country.id).unwrap();
        assert_eq!(db::list_countries(&state.conn).unwrap().len(), 1);
        assert_eq!(db::list_places(&state.conn, &country.id).unwrap().len(), 1);
        assert_eq!(db::list_country_photos(&state.conn, &country.id, 8).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reorder_countries() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let a = db::create_country(&state.conn, "Japan", None).unwrap();
        let b = db::create_country(&state.conn, "Iceland", None).unwrap();
        db::reorder(&state.conn, "countries", &[b.id.clone(), a.id.clone()]).unwrap();
        let listed = db::list_countries(&state.conn).unwrap();
        assert_eq!(listed[0].id, b.id);
        assert_eq!(listed[1].id, a.id);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn trip_plan_budget_and_tasks() {
        let dir = std::env::temp_dir().join(format!("bucket-test-{}", Uuid::new_v4()));
        let state = init_library_at(dir.clone()).unwrap();
        let country = db::create_country(&state.conn, "Indonesia", Some("ID")).unwrap();
        let falls = db::create_place(&state.conn, &country.id, "Tumpak Sewu", None).unwrap();
        let trip = db::create_trip(&state.conn, "Java").unwrap();
        let dated = db::update_trip(
            &state.conn,
            &trip.id,
            None,
            Some("June 2027"),
            Some("INR"),
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(dated.year, Some(2027));
        assert_eq!(dated.currency.as_deref(), Some("INR"));
        let someday = db::update_trip(
            &state.conn,
            &trip.id,
            None,
            Some("dry season"),
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(someday.year, None);

        db::add_trip_stop(&state.conn, &trip.id, &falls.id).unwrap();
        assert!(db::add_trip_stop(&state.conn, &trip.id, &falls.id).is_err());
        let priced = db::set_trip_cost(&state.conn, &trip.id, "flights", Some(40000)).unwrap();
        assert_eq!(priced.trip.total, 40000);
        assert_eq!(priced.costs.len(), 11);
        assert!(db::set_trip_cost(&state.conn, &trip.id, "rafting", Some(10)).is_err());
        db::set_trip_cost(&state.conn, &trip.id, "hotels", Some(12000)).unwrap();
        let cleared = db::set_trip_cost(&state.conn, &trip.id, "flights", None).unwrap();
        assert_eq!(cleared.trip.total, 12000);

        db::update_trip(&state.conn, &trip.id, None, None, None, None, Some("done"), true).unwrap();
        assert_eq!(db::get_place(&state.conn, &falls.id).unwrap().status, "been");

        let task = db::add_place_task(&state.conn, &falls.id, "Sunrise viewpoint").unwrap();
        let done = db::set_place_task(&state.conn, &task.id, None, Some(true)).unwrap();
        assert!(done.done);
        let hits = db::search(&state.conn, "Java").unwrap();
        assert!(hits.iter().any(|hit| hit.kind == "trip"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
