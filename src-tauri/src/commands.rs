use crate::capture;
use crate::db::{self, AppState, DbError};
use crate::export;
use crate::images;
use crate::models::{Country, ImageRecord, InboxItem, LinkRecord, Place, SearchHit};
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

type SharedState = Mutex<AppState>;

fn map_err(e: DbError) -> String {
    e.to_string()
}

fn lock<'a>(state: &'a State<'_, SharedState>) -> Result<std::sync::MutexGuard<'a, AppState>, String> {
    state.lock().map_err(|_| "App state lock poisoned".to_string())
}

#[tauri::command]
pub fn list_countries(state: State<'_, SharedState>) -> Result<Vec<Country>, String> {
    db::list_countries(&lock(&state)?.conn).map_err(map_err)
}

#[tauri::command]
pub fn create_country(
    state: State<'_, SharedState>,
    name: String,
    iso: Option<String>,
) -> Result<Country, String> {
    db::create_country(&lock(&state)?.conn, &name, iso.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn update_country(
    state: State<'_, SharedState>,
    country_id: String,
    name: Option<String>,
    iso: Option<String>,
) -> Result<Country, String> {
    db::update_country(
        &lock(&state)?.conn,
        &country_id,
        name.as_deref(),
        iso.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_country(state: State<'_, SharedState>, country_id: String) -> Result<(), String> {
    db::delete_country(&lock(&state)?.conn, &country_id).map_err(map_err)
}

#[tauri::command]
pub fn get_library_path(state: State<'_, SharedState>) -> Result<String, String> {
    Ok(lock(&state)?.library_root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn reveal_library(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let root = lock(&state)?.library_root.clone();
    app.opener()
        .open_path(root.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_places(state: State<'_, SharedState>, country_id: String) -> Result<Vec<Place>, String> {
    db::list_places(&lock(&state)?.conn, &country_id).map_err(map_err)
}

#[tauri::command]
pub fn get_place(state: State<'_, SharedState>, place_id: String) -> Result<Place, String> {
    db::get_place(&lock(&state)?.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn create_place(
    state: State<'_, SharedState>,
    country_id: String,
    name: String,
    status: Option<String>,
) -> Result<Place, String> {
    db::create_place(&lock(&state)?.conn, &country_id, &name, status.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn update_place(
    state: State<'_, SharedState>,
    place_id: String,
    name: Option<String>,
    notes: Option<String>,
    status: Option<String>,
) -> Result<Place, String> {
    db::update_place(
        &lock(&state)?.conn,
        &place_id,
        name.as_deref(),
        notes.as_deref(),
        status.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_place(state: State<'_, SharedState>, place_id: String) -> Result<(), String> {
    let state = lock(&state)?;
    db::delete_place(&state.conn, &place_id).map_err(map_err)?;
    let dir = state.library_root.join("library/images").join(&place_id);
    let _ = std::fs::remove_dir_all(dir);
    Ok(())
}

#[tauri::command]
pub fn list_images(state: State<'_, SharedState>, place_id: String) -> Result<Vec<ImageRecord>, String> {
    db::list_images(&lock(&state)?.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn import_image_bytes(
    state: State<'_, SharedState>,
    place_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> Result<ImageRecord, String> {
    let state = lock(&state)?;
    images::import_image_bytes(&state, &place_id, &filename, bytes).map_err(map_err)
}

#[tauri::command]
pub fn import_image_path(
    state: State<'_, SharedState>,
    place_id: String,
    source_path: String,
) -> Result<ImageRecord, String> {
    let state = lock(&state)?;
    images::import_image_path(&state, &place_id, &source_path).map_err(map_err)
}

#[tauri::command]
pub fn list_links(
    state: State<'_, SharedState>,
    place_id: Option<String>,
    country_id: Option<String>,
) -> Result<Vec<LinkRecord>, String> {
    db::list_links(
        &lock(&state)?.conn,
        place_id.as_deref(),
        country_id.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn add_link(
    state: State<'_, SharedState>,
    url: String,
    title: Option<String>,
    place_id: Option<String>,
    country_id: Option<String>,
) -> Result<LinkRecord, String> {
    let source = if capture::is_youtube(&url) {
        "youtube"
    } else {
        "web"
    };
    db::insert_link(
        &lock(&state)?.conn,
        &url,
        title.as_deref(),
        None,
        source,
        place_id.as_deref(),
        country_id.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_link(state: State<'_, SharedState>, link_id: String) -> Result<(), String> {
    db::delete_link(&lock(&state)?.conn, &link_id).map_err(map_err)
}

#[tauri::command]
pub fn list_inbox(state: State<'_, SharedState>) -> Result<Vec<InboxItem>, String> {
    db::list_inbox(&lock(&state)?.conn).map_err(map_err)
}

#[tauri::command]
pub fn inbox_count(state: State<'_, SharedState>) -> Result<i32, String> {
    db::inbox_count(&lock(&state)?.conn).map_err(map_err)
}

#[tauri::command]
pub fn capture_inbox_url(state: State<'_, SharedState>, url: String) -> Result<InboxItem, String> {
    let trimmed = url.trim().to_string();
    if trimmed.is_empty() {
        return Err("Paste a URL first.".into());
    }

    let mut title = None;
    let mut thumb_url = None;
    let mut source = "web";

    if capture::is_youtube(&trimmed) {
        source = "youtube";
        if let Ok((fetched_title, fetched_thumb)) = capture::fetch_youtube_meta(&trimmed) {
            title = fetched_title;
            thumb_url = fetched_thumb;
        }
    }

    let state = lock(&state)?;
    let mut thumb_relpath = None;
    if let Some(thumb_url) = thumb_url.as_deref() {
        if let Ok(rel) = capture::download_thumb(&state.library_root, thumb_url) {
            thumb_relpath = Some(rel);
        }
    }

    db::insert_inbox(
        &state.conn,
        &trimmed,
        title.as_deref(),
        thumb_relpath.as_deref(),
        None,
        source,
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn capture_inbox_image(
    state: State<'_, SharedState>,
    filename: String,
    bytes: Vec<u8>,
) -> Result<InboxItem, String> {
    let state = lock(&state)?;
    let rel = images::save_inbox_bytes(&state, &filename, bytes).map_err(map_err)?;
    db::insert_inbox(
        &state.conn,
        "",
        Some(&filename),
        Some(&rel),
        Some(&rel),
        "image",
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn file_inbox(
    state: State<'_, SharedState>,
    inbox_id: String,
    country_id: String,
    place_id: Option<String>,
    new_place_name: Option<String>,
) -> Result<InboxItem, String> {
    let state = lock(&state)?;
    let item = db::get_inbox(&state.conn, &inbox_id).map_err(map_err)?;
    if item.filed_at.is_some() {
        return Err("Already filed.".into());
    }
    db::get_country(&state.conn, &country_id).map_err(map_err)?;

    let mut resolved_place = place_id;
    if let Some(name) = new_place_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let place = db::create_place(&state.conn, &country_id, name, None).map_err(map_err)?;
        resolved_place = Some(place.id);
    }

    if !item.url.is_empty() {
        db::insert_link(
            &state.conn,
            &item.url,
            item.title.as_deref(),
            item.thumb_relpath.as_deref(),
            &item.source,
            resolved_place.as_deref(),
            Some(&country_id),
        )
        .map_err(map_err)?;
    }

    if let (Some(place_id), Some(image_rel)) = (resolved_place.as_deref(), item.image_relpath.as_deref())
    {
        let abs = db::abs_from_relpath(&state.library_root, image_rel);
        let _ = images::import_image_path(&state, place_id, &abs.to_string_lossy());
    }

    db::mark_inbox_filed(&state.conn, &inbox_id).map_err(map_err)
}

#[tauri::command]
pub fn delete_inbox(state: State<'_, SharedState>, inbox_id: String) -> Result<(), String> {
    db::delete_inbox(&lock(&state)?.conn, &inbox_id).map_err(map_err)
}

#[tauri::command]
pub fn search(state: State<'_, SharedState>, query: String) -> Result<Vec<SearchHit>, String> {
    db::search(&lock(&state)?.conn, &query).map_err(map_err)
}

#[tauri::command]
pub fn export_library_zip(state: State<'_, SharedState>, dest_path: String) -> Result<(), String> {
    let root = lock(&state)?.library_root.clone();
    export::zip_library(&root, std::path::Path::new(&dest_path)).map_err(map_err)
}
