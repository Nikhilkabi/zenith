use crate::db::{self, AppState, DbError};
use crate::images;
use crate::models::{Country, ImageRecord, Place};
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

type SharedState = Mutex<AppState>;

fn map_err(e: DbError) -> String {
    e.to_string()
}

#[tauri::command]
pub fn list_countries(state: State<'_, SharedState>) -> Result<Vec<Country>, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::list_countries(&state.conn).map_err(map_err)
}

#[tauri::command]
pub fn create_country(
    state: State<'_, SharedState>,
    name: String,
    iso: Option<String>,
) -> Result<Country, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::create_country(&state.conn, &name, iso.as_deref()).map_err(map_err)
}

#[tauri::command]
pub fn get_library_path(state: State<'_, SharedState>) -> Result<String, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    Ok(state.library_root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn reveal_library(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
) -> Result<(), String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    app.opener()
        .open_path(state.library_root.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_places(state: State<'_, SharedState>, country_id: String) -> Result<Vec<Place>, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::list_places(&state.conn, &country_id).map_err(map_err)
}

#[tauri::command]
pub fn get_place(state: State<'_, SharedState>, place_id: String) -> Result<Place, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::get_place(&state.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn create_place(
    state: State<'_, SharedState>,
    country_id: String,
    name: String,
    status: Option<String>,
) -> Result<Place, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::create_place(
        &state.conn,
        &country_id,
        &name,
        status.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn update_place(
    state: State<'_, SharedState>,
    place_id: String,
    name: Option<String>,
    notes: Option<String>,
    status: Option<String>,
) -> Result<Place, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::update_place(
        &state.conn,
        &place_id,
        name.as_deref(),
        notes.as_deref(),
        status.as_deref(),
    )
    .map_err(map_err)
}

#[tauri::command]
pub fn delete_place(state: State<'_, SharedState>, place_id: String) -> Result<(), String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::delete_place(&state.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn list_images(state: State<'_, SharedState>, place_id: String) -> Result<Vec<ImageRecord>, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    db::list_images(&state.conn, &place_id).map_err(map_err)
}

#[tauri::command]
pub fn import_image_bytes(
    state: State<'_, SharedState>,
    place_id: String,
    filename: String,
    bytes: Vec<u8>,
) -> Result<ImageRecord, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    images::import_image_bytes(&state, &place_id, &filename, bytes).map_err(map_err)
}

#[tauri::command]
pub fn import_image_path(
    state: State<'_, SharedState>,
    place_id: String,
    source_path: String,
) -> Result<ImageRecord, String> {
    let state = state.lock().map_err(|_| "App state lock poisoned".to_string())?;
    images::import_image_path(&state, &place_id, &source_path).map_err(map_err)
}
