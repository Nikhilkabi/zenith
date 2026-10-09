use crate::capture;
use crate::covers;
use crate::db::{self, AppState, DbError};
use crate::export;
use crate::images;
use crate::models::{
    Country, CoverCredit, CoverPage, Goal, ImageRecord, LinkRecord, Place, PlaceTask, SearchHit,
    TrashItem, Trip, TripDetail,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

pub type SharedState = Arc<Mutex<AppState>>;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn parse_id(id: &str, what: &str) -> Result<(), String> {
    Uuid::parse_str(id).map_err(|_| format!("Invalid {what} id"))?;
    Ok(())
}

fn parse_id_opt(id: Option<&str>, what: &str) -> Result<(), String> {
    if let Some(id) = id {
        parse_id(id, what)?;
    }
    Ok(())
}

fn clone_state(state: &State<'_, SharedState>) -> SharedState {
    state.inner().clone()
}

fn lock_arc(state: &SharedState) -> Result<std::sync::MutexGuard<'_, AppState>, String> {
    state.lock().map_err(|_| "App state lock poisoned".to_string())
}

async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn list_countries(state: State<'_, SharedState>) -> Result<Vec<Country>, String> {
    db::list_countries(&lock_arc(state.inner())?.conn).map_err(map_err)
}

#[tauri::command]
pub fn create_country(
    state: State<'_, SharedState>,
    name: String,
    iso: Option<String>,
) -> Result<Country, String> {
    db::create_country(&lock_arc(state.inner())?.conn, &name, iso.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn update_country(
    state: State<'_, SharedState>,
    country_id: String,
    name: Option<String>,
    iso: Option<String>,
) -> Result<Country, String> {
    parse_id(&country_id, "country")?;
    db::update_country(
        &lock_arc(state.inner())?.conn,
        &country_id,
        name.as_deref(),
        iso.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_country(state: State<'_, SharedState>, country_id: String) -> Result<(), String> {
    parse_id(&country_id, "country")?;
    let guard = lock_arc(state.inner())?;
    db::delete_country_with_files(&guard, &country_id).map_err(map_err)
}

#[tauri::command]
pub fn get_library_path(state: State<'_, SharedState>) -> Result<String, String> {
    Ok(lock_arc(state.inner())?
        .library_root
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub fn reveal_library(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let root = lock_arc(state.inner())?.library_root.clone();
    app.opener()
        .open_path(root.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed = capture::require_http_url(&url).map_err(map_err)?;
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_places(state: State<'_, SharedState>, country_id: String) -> Result<Vec<Place>, String> {
    parse_id(&country_id, "country")?;
    db::list_places(&lock_arc(state.inner())?.conn, &country_id).map_err(map_err)
}

#[tauri::command]
pub fn list_all_places(state: State<'_, SharedState>) -> Result<Vec<Place>, String> {
    db::list_all_places(&lock_arc(state.inner())?.conn).map_err(map_err)
}

#[tauri::command]
pub fn get_place(state: State<'_, SharedState>, place_id: String) -> Result<Place, String> {
    parse_id(&place_id, "place")?;
    db::get_place(&lock_arc(state.inner())?.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn create_place(
    state: State<'_, SharedState>,
    country_id: String,
    name: String,
    status: Option<String>,
) -> Result<Place, String> {
    parse_id(&country_id, "country")?;
    db::create_place(&lock_arc(state.inner())?.conn, &country_id, &name, status.as_deref())
        .map_err(map_err)
}

#[tauri::command]
pub fn update_place(
    state: State<'_, SharedState>,
    place_id: String,
    name: Option<String>,
    notes: Option<String>,
    status: Option<String>,
    when_text: Option<String>,
) -> Result<Place, String> {
    parse_id(&place_id, "place")?;
    db::update_place(
        &lock_arc(state.inner())?.conn,
        &place_id,
        name.as_deref(),
        notes.as_deref(),
        status.as_deref(),
        when_text.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_place(state: State<'_, SharedState>, place_id: String) -> Result<(), String> {
    parse_id(&place_id, "place")?;
    let state = lock_arc(state.inner())?;
    db::delete_place(&state.conn, &place_id).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn list_images(state: State<'_, SharedState>, place_id: String) -> Result<Vec<ImageRecord>, String> {
    parse_id(&place_id, "place")?;
    db::list_images(&lock_arc(state.inner())?.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn delete_image(state: State<'_, SharedState>, image_id: String) -> Result<(), String> {
    parse_id(&image_id, "image")?;
    let guard = lock_arc(state.inner())?;
    db::delete_image(&guard, &image_id).map_err(map_err)
}

#[tauri::command]
pub fn set_place_cover(
    state: State<'_, SharedState>,
    place_id: String,
    image_id: String,
) -> Result<Place, String> {
    parse_id(&place_id, "place")?;
    parse_id(&image_id, "image")?;
    db::set_place_cover(&lock_arc(state.inner())?.conn, &place_id, &image_id).map_err(map_err)
}

#[tauri::command]
pub fn set_country_cover(
    state: State<'_, SharedState>,
    country_id: String,
    image_id: String,
) -> Result<Country, String> {
    parse_id(&country_id, "country")?;
    parse_id(&image_id, "image")?;
    db::set_country_cover(&lock_arc(state.inner())?.conn, &country_id, &image_id).map_err(map_err)
}

#[tauri::command]
pub async fn import_image_bytes(
    state: State<'_, SharedState>,
    place_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> Result<ImageRecord, String> {
    parse_id(&place_id, "place")?;
    let state = clone_state(&state);
    blocking(move || {
        let guard = lock_arc(&state)?;
        images::import_image_bytes(&guard, &place_id, &filename, bytes).map_err(map_err)
    })
    .await
}

#[tauri::command]
pub async fn import_image_path(
    state: State<'_, SharedState>,
    place_id: String,
    source_path: String,
) -> Result<ImageRecord, String> {
    parse_id(&place_id, "place")?;
    let state = clone_state(&state);
    blocking(move || {
        let guard = lock_arc(&state)?;
        images::import_image_path(&guard, &place_id, &source_path).map_err(map_err)
    })
    .await
}

#[tauri::command]
pub fn list_links(
    state: State<'_, SharedState>,
    place_id: Option<String>,
    country_id: Option<String>,
) -> Result<Vec<LinkRecord>, String> {
    parse_id_opt(place_id.as_deref(), "place")?;
    parse_id_opt(country_id.as_deref(), "country")?;
    let mut links = db::list_links(
        &lock_arc(state.inner())?.conn,
        place_id.as_deref(),
        country_id.as_deref(),
    )
    .map_err(map_err)?;
    for link in &mut links {
        fill_youtube_link(state.inner(), link);
    }
    Ok(links)
}

#[tauri::command]
pub fn add_link(
    state: State<'_, SharedState>,
    url: String,
    title: Option<String>,
    place_id: Option<String>,
    country_id: Option<String>,
) -> Result<LinkRecord, String> {
    parse_id_opt(place_id.as_deref(), "place")?;
    parse_id_opt(country_id.as_deref(), "country")?;
    let parsed = capture::require_http_url(&url).map_err(map_err)?;
    let youtube = capture::is_youtube(parsed.as_str());
    {
        let guard = lock_arc(state.inner())?;
        if db::link_url_exists(&guard.conn, parsed.as_str()).map_err(map_err)? {
            return Err("That link is already saved".into());
        }
    }
    let preview = if youtube {
        capture::youtube_preview(parsed.as_str()).ok()
    } else {
        None
    };
    let named = title.filter(|value| !value.trim().is_empty());
    let chosen_title = named.as_deref().or_else(|| preview.as_ref().map(|item| item.title.as_str()));
    let guard = lock_arc(state.inner())?;
    let thumb_saved = preview.as_ref().and_then(|item| {
        item.thumb.as_deref().and_then(|bytes| save_link_thumb(&guard.library_root, bytes))
    });
    let thumb_rel = thumb_saved.as_ref().map(|(rel, _)| rel.as_str());
    let source = if youtube { "youtube" } else { "web" };
    match db::insert_link(
        &guard.conn,
        parsed.as_str(),
        chosen_title,
        thumb_rel,
        source,
        place_id.as_deref(),
        country_id.as_deref(),
    ) {
        Ok(record) => Ok(record),
        Err(err) => {
            if let Some((_, abs)) = thumb_saved {
                let _ = fs::remove_file(abs);
            }
            Err(map_err(err))
        }
    }
}

fn fill_youtube_link(state: &SharedState, link: &mut LinkRecord) {
    if !capture::is_youtube(&link.url) || (link.title.is_some() && link.thumb_relpath.is_some()) {
        return;
    }
    let Ok(preview) = capture::youtube_preview(&link.url) else {
        return;
    };
    let root = match lock_arc(state) {
        Ok(guard) => guard.library_root.clone(),
        Err(_) => return,
    };
    let thumb_saved = if link.thumb_relpath.is_none() {
        preview.thumb.as_deref().and_then(|bytes| save_link_thumb(&root, bytes))
    } else {
        None
    };
    let title = if link.title.is_none() {
        Some(preview.title.as_str())
    } else {
        None
    };
    let thumb_rel = thumb_saved.as_ref().map(|(rel, _)| rel.as_str());
    let saved = lock_arc(state)
        .and_then(|guard| db::set_link_preview(&guard.conn, &link.id, title, thumb_rel).map_err(map_err));
    if saved.is_err() {
        if let Some((_, abs)) = thumb_saved {
            let _ = fs::remove_file(abs);
        }
        return;
    }
    if link.title.is_none() {
        link.title = Some(preview.title);
    }
    if link.thumb_relpath.is_none() {
        link.thumb_relpath = thumb_saved.map(|(rel, _)| rel);
    }
}

fn save_link_thumb(library_root: &Path, bytes: &[u8]) -> Option<(String, PathBuf)> {
    let dir = library_root.join("library/link-thumbs");
    fs::create_dir_all(&dir).ok()?;
    let abs = dir.join(format!("{}.jpg", Uuid::new_v4()));
    images::write_cover_thumb(&abs, bytes).ok()?;
    let rel = db::relpath_from_abs(library_root, &abs).ok()?;
    Some((rel, abs))
}

#[tauri::command]
pub fn delete_link(state: State<'_, SharedState>, link_id: String) -> Result<(), String> {
    parse_id(&link_id, "link")?;
    let guard = lock_arc(state.inner())?;
    db::delete_link_with_files(&guard, &link_id).map_err(map_err)
}

#[tauri::command]
pub fn search(state: State<'_, SharedState>, query: String) -> Result<Vec<SearchHit>, String> {
    db::search(&lock_arc(state.inner())?.conn, &query).map_err(map_err)
}

#[tauri::command]
pub async fn export_library_zip(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let dest = blocking({
        let app = app.clone();
        move || {
            Ok(app
                .dialog()
                .file()
                .add_filter("Zip", &["zip"])
                .set_file_name("Zenith-backup.zip")
                .blocking_save_file())
        }
    })
    .await?;

    let Some(dest) = dest else {
        return Ok(());
    };
    let dest = dest.into_path().map_err(|e| e.to_string())?;
    let root = lock_arc(state.inner())?.library_root.clone();
    blocking(move || export::zip_library(&root, &dest).map_err(map_err)).await
}

#[tauri::command]
pub fn sweep_orphans(state: State<'_, SharedState>) -> Result<u32, String> {
    let guard = lock_arc(state.inner())?;
    db::sweep_orphans(&guard).map_err(map_err)
}

#[tauri::command]
pub fn list_country_photos(
    state: State<'_, SharedState>,
    country_id: String,
) -> Result<Vec<ImageRecord>, String> {
    parse_id(&country_id, "country")?;
    db::list_country_photos(&lock_arc(state.inner())?.conn, &country_id, 8).map_err(map_err)
}

#[tauri::command]
pub fn reorder_countries(state: State<'_, SharedState>, ids: Vec<String>) -> Result<(), String> {
    for id in &ids {
        parse_id(id, "country")?;
    }
    db::reorder(&lock_arc(state.inner())?.conn, "countries", &ids).map_err(map_err)
}

#[tauri::command]
pub fn reorder_places(state: State<'_, SharedState>, ids: Vec<String>) -> Result<(), String> {
    for id in &ids {
        parse_id(id, "place")?;
    }
    db::reorder(&lock_arc(state.inner())?.conn, "places", &ids).map_err(map_err)
}

#[tauri::command]
pub fn update_image_caption(
    state: State<'_, SharedState>,
    image_id: String,
    caption: Option<String>,
) -> Result<ImageRecord, String> {
    parse_id(&image_id, "image")?;
    db::update_image_caption(
        &lock_arc(state.inner())?.conn,
        &image_id,
        caption.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn list_trash(state: State<'_, SharedState>) -> Result<Vec<TrashItem>, String> {
    db::list_trash(&lock_arc(state.inner())?.conn).map_err(map_err)
}

#[tauri::command]
pub fn restore_trash(
    state: State<'_, SharedState>,
    kind: String,
    id: String,
) -> Result<(), String> {
    parse_id(&id, "item")?;
    db::restore_trash(&lock_arc(state.inner())?.conn, &kind, &id).map_err(map_err)
}

#[tauri::command]
pub fn purge_trash(state: State<'_, SharedState>) -> Result<u32, String> {
    let guard = lock_arc(state.inner())?;
    db::purge_trash(&guard, 30).map_err(map_err)
}

#[tauri::command]
pub fn drop_on_country(
    state: State<'_, SharedState>,
    country_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> Result<ImageRecord, String> {
    parse_id(&country_id, "country")?;
    let state = lock_arc(state.inner())?;
    let place = db::ensure_place(&state.conn, &country_id, "Unsorted").map_err(map_err)?;
    images::import_image_bytes(&state, &place.id, &filename, bytes).map_err(map_err)
}

#[tauri::command]
pub async fn import_library_zip(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
) -> Result<u32, String> {
    let dest = blocking({
        let app = app.clone();
        move || {
            Ok(app
                .dialog()
                .file()
                .add_filter("Zip", &["zip"])
                .blocking_pick_file())
        }
    })
    .await?;
    let Some(dest) = dest else {
        return Ok(0);
    };
    let dest = dest.into_path().map_err(|e| e.to_string())?;
    let root = lock_arc(state.inner())?.library_root.clone();
    blocking(move || export::import_library_zip(&root, &dest).map_err(map_err)).await
}

fn parse_cover_target(target: &str, id: &str) -> Result<(), String> {
    match target {
        "goal" => parse_id(id, "goal"),
        "trip" => parse_id(id, "trip"),
        "country" => parse_id(id, "country"),
        "place" => parse_id(id, "place"),
        _ => Err("Unknown cover target".into()),
    }
}

#[tauri::command]
pub fn list_goals(state: State<'_, SharedState>) -> Result<Vec<Goal>, String> {
    db::list_goals(&lock_arc(state.inner())?.conn).map_err(map_err)
}

#[tauri::command]
pub fn get_goal(state: State<'_, SharedState>, goal_id: String) -> Result<Goal, String> {
    parse_id(&goal_id, "goal")?;
    db::get_goal(&lock_arc(state.inner())?.conn, &goal_id).map_err(map_err)
}

#[tauri::command]
pub fn create_goal(state: State<'_, SharedState>, name: String) -> Result<Goal, String> {
    db::create_goal(&lock_arc(state.inner())?.conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn update_goal(
    state: State<'_, SharedState>,
    goal_id: String,
    name: Option<String>,
    notes: Option<String>,
    status: Option<String>,
) -> Result<Goal, String> {
    parse_id(&goal_id, "goal")?;
    db::update_goal(
        &lock_arc(state.inner())?.conn,
        &goal_id,
        name.as_deref(),
        notes.as_deref(),
        status.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_goal(state: State<'_, SharedState>, goal_id: String) -> Result<(), String> {
    parse_id(&goal_id, "goal")?;
    db::delete_goal(&lock_arc(state.inner())?.conn, &goal_id).map_err(map_err)
}

#[tauri::command]
pub fn list_trips(state: State<'_, SharedState>) -> Result<Vec<Trip>, String> {
    db::list_trips(&lock_arc(state.inner())?.conn).map_err(map_err)
}

#[tauri::command]
pub fn get_trip(state: State<'_, SharedState>, trip_id: String) -> Result<TripDetail, String> {
    parse_id(&trip_id, "trip")?;
    db::trip_detail(&lock_arc(state.inner())?.conn, &trip_id).map_err(map_err)
}

#[tauri::command]
pub fn create_trip(state: State<'_, SharedState>, name: String) -> Result<Trip, String> {
    db::create_trip(&lock_arc(state.inner())?.conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn update_trip(
    state: State<'_, SharedState>,
    trip_id: String,
    name: Option<String>,
    when_text: Option<String>,
    currency: Option<String>,
    notes: Option<String>,
    status: Option<String>,
    mark_places: Option<bool>,
) -> Result<Trip, String> {
    parse_id(&trip_id, "trip")?;
    db::update_trip(
        &lock_arc(state.inner())?.conn,
        &trip_id,
        name.as_deref(),
        when_text.as_deref(),
        currency.as_deref(),
        notes.as_deref(),
        status.as_deref(),
        mark_places.unwrap_or(false),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_trip(state: State<'_, SharedState>, trip_id: String) -> Result<(), String> {
    parse_id(&trip_id, "trip")?;
    db::delete_trip(&lock_arc(state.inner())?.conn, &trip_id).map_err(map_err)
}

#[tauri::command]
pub fn add_trip_stop(
    state: State<'_, SharedState>,
    trip_id: String,
    place_id: String,
) -> Result<TripDetail, String> {
    parse_id(&trip_id, "trip")?;
    parse_id(&place_id, "place")?;
    let guard = lock_arc(state.inner())?;
    db::add_trip_stop(&guard.conn, &trip_id, &place_id).map_err(map_err)?;
    db::trip_detail(&guard.conn, &trip_id).map_err(map_err)
}

#[tauri::command]
pub fn remove_trip_stop(
    state: State<'_, SharedState>,
    trip_id: String,
    place_id: String,
) -> Result<TripDetail, String> {
    parse_id(&trip_id, "trip")?;
    parse_id(&place_id, "place")?;
    let guard = lock_arc(state.inner())?;
    db::remove_trip_stop(&guard.conn, &trip_id, &place_id).map_err(map_err)?;
    db::trip_detail(&guard.conn, &trip_id).map_err(map_err)
}

#[tauri::command]
pub fn reorder_trip_stops(
    state: State<'_, SharedState>,
    trip_id: String,
    place_ids: Vec<String>,
) -> Result<TripDetail, String> {
    parse_id(&trip_id, "trip")?;
    for id in &place_ids {
        parse_id(id, "place")?;
    }
    let guard = lock_arc(state.inner())?;
    db::reorder_trip_stops(&guard.conn, &trip_id, &place_ids).map_err(map_err)?;
    db::trip_detail(&guard.conn, &trip_id).map_err(map_err)
}

#[tauri::command]
pub fn set_trip_cost(
    state: State<'_, SharedState>,
    trip_id: String,
    category: String,
    amount: Option<i64>,
) -> Result<TripDetail, String> {
    parse_id(&trip_id, "trip")?;
    db::set_trip_cost(&lock_arc(state.inner())?.conn, &trip_id, &category, amount).map_err(map_err)
}

#[tauri::command]
pub fn list_place_tasks(state: State<'_, SharedState>, place_id: String) -> Result<Vec<PlaceTask>, String> {
    parse_id(&place_id, "place")?;
    db::list_place_tasks(&lock_arc(state.inner())?.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn add_place_task(
    state: State<'_, SharedState>,
    place_id: String,
    body: String,
) -> Result<PlaceTask, String> {
    parse_id(&place_id, "place")?;
    db::add_place_task(&lock_arc(state.inner())?.conn, &place_id, &body).map_err(map_err)
}

#[tauri::command]
pub fn set_place_task(
    state: State<'_, SharedState>,
    task_id: String,
    body: Option<String>,
    done: Option<bool>,
) -> Result<PlaceTask, String> {
    parse_id(&task_id, "task")?;
    db::set_place_task(
        &lock_arc(state.inner())?.conn,
        &task_id,
        body.as_deref(),
        done,
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_place_task(state: State<'_, SharedState>, task_id: String) -> Result<(), String> {
    parse_id(&task_id, "task")?;
    db::delete_place_task(&lock_arc(state.inner())?.conn, &task_id).map_err(map_err)
}

#[tauri::command]
pub fn reorder_goals(state: State<'_, SharedState>, ids: Vec<String>) -> Result<(), String> {
    for id in &ids {
        parse_id(id, "goal")?;
    }
    db::reorder_goals(&lock_arc(state.inner())?.conn, &ids).map_err(map_err)
}

#[tauri::command]
pub fn cover_credit(
    state: State<'_, SharedState>,
    relpath: String,
) -> Result<Option<CoverCredit>, String> {
    if relpath.contains("..") || !relpath.starts_with("library/") {
        return Err("Invalid cover path".into());
    }
    let found = db::cover_credit(&lock_arc(state.inner())?.conn, &relpath).map_err(map_err)?;
    Ok(found.map(|(credit, credit_url)| CoverCredit { credit, credit_url }))
}

#[tauri::command]
pub fn get_photo_sources() -> Result<covers::PhotoSources, String> {
    covers::load_sources().map_err(map_err)
}

#[tauri::command]
pub fn set_photo_sources(pexels: String, pixabay: String) -> Result<covers::PhotoSources, String> {
    covers::save_sources(&pexels, &pixabay).map_err(map_err)
}

#[tauri::command]
pub async fn import_stock_photo(
    state: State<'_, SharedState>,
    place_id: String,
    image_id: String,
) -> Result<ImageRecord, String> {
    parse_id(&place_id, "place")?;
    let state = clone_state(&state);
    blocking(move || {
        let guard = lock_arc(&state)?;
        covers::import_stock_photo(&guard, &place_id, &image_id).map_err(map_err)
    })
    .await
}

#[tauri::command]
pub async fn search_covers(
    state: State<'_, SharedState>,
    query: String,
    page: u32,
) -> Result<CoverPage, String> {
    let root = lock_arc(state.inner())?.library_root.clone();
    blocking(move || covers::search(&root, &query, page).map_err(map_err)).await
}

#[tauri::command]
pub async fn apply_stock_cover(
    state: State<'_, SharedState>,
    target: String,
    owner_id: String,
    image_id: String,
    focus_x: f64,
    focus_y: f64,
) -> Result<(), String> {
    parse_cover_target(&target, &owner_id)?;
    let state = clone_state(&state);
    blocking(move || {
        let guard = lock_arc(&state)?;
        covers::apply_stock(&guard, &target, &owner_id, &image_id, focus_x, focus_y).map_err(map_err)
    })
    .await
}

#[tauri::command]
pub async fn apply_file_cover(
    state: State<'_, SharedState>,
    target: String,
    owner_id: String,
    filename: String,
    bytes: Vec<u8>,
    focus_x: f64,
    focus_y: f64,
) -> Result<(), String> {
    parse_cover_target(&target, &owner_id)?;
    let state = clone_state(&state);
    blocking(move || {
        let guard = lock_arc(&state)?;
        covers::apply_file(&guard, &target, &owner_id, &filename, &bytes, focus_x, focus_y)
            .map_err(map_err)
    })
    .await
}

#[tauri::command]
pub fn set_cover_focus(
    state: State<'_, SharedState>,
    target: String,
    owner_id: String,
    focus_x: f64,
    focus_y: f64,
) -> Result<(), String> {
    parse_cover_target(&target, &owner_id)?;
    db::set_cover_focus(
        &lock_arc(state.inner())?.conn,
        &target,
        &owner_id,
        focus_x,
        focus_y,
    )
    .map_err(map_err)
}
