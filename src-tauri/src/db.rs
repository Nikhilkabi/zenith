use crate::models::{
    Country, Goal, ImageRecord, LinkRecord, Place, PlaceTask, SearchHit, TrashItem, Trip, TripCost,
    TripDetail, TripStop,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

impl DbError {
    pub(crate) fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

pub struct AppState {
    pub library_root: PathBuf,
    pub conn: Connection,
}

const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("migrations/001_initial.sql")),
    (2, include_str!("migrations/002_enrich.sql")),
    (3, include_str!("migrations/003_goals.sql")),
    (4, include_str!("migrations/004_cover_focus.sql")),
    (5, include_str!("migrations/005_trips.sql")),
];

pub fn init_library() -> Result<AppState, DbError> {
    let docs = dirs::document_dir().ok_or_else(|| DbError::msg("Could not find Documents folder"))?;
    init_library_at(docs.join("Bucket"))
}

pub fn init_library_at(library_root: PathBuf) -> Result<AppState, DbError> {
    fs::create_dir_all(library_root.join("library"))?;

    let db_path = library_root.join("db.sqlite");
    let conn = Connection::open(&db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    run_migrations(&conn)?;

    Ok(AppState { library_root, conn })
}

fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    for (version, sql) in MIGRATIONS {
        let applied: Option<i32> = conn
            .query_row(
                "SELECT version FROM migrations WHERE version = ?1",
                params![version],
                |row| row.get(0),
            )
            .ok();

        if applied.is_some() {
            continue;
        }

        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, Utc::now().to_rfc3339()],
        )?;
    }

    Ok(())
}

const COUNTRY_SELECT: &str = "SELECT c.id, c.name, c.iso,
                COALESCE(
                  c.cover_relpath,
                  (SELECT p.cover_relpath FROM places p
                   WHERE p.country_id = c.id AND p.cover_relpath IS NOT NULL
                   ORDER BY p.sort ASC, p.name ASC LIMIT 1)
                ) AS cover_relpath,
                CASE WHEN c.cover_relpath IS NOT NULL THEN c.cover_x
                     ELSE COALESCE(
                       (SELECT p.cover_x FROM places p
                        WHERE p.country_id = c.id AND p.cover_relpath IS NOT NULL
                        ORDER BY p.sort ASC, p.name ASC LIMIT 1),
                       50)
                END AS cover_x,
                CASE WHEN c.cover_relpath IS NOT NULL THEN c.cover_y
                     ELSE COALESCE(
                       (SELECT p.cover_y FROM places p
                        WHERE p.country_id = c.id AND p.cover_relpath IS NOT NULL
                        ORDER BY p.sort ASC, p.name ASC LIMIT 1),
                       50)
                END AS cover_y,
                c.sort, c.created_at,
                (SELECT COUNT(*) FROM places p WHERE p.country_id = c.id AND p.deleted_at IS NULL) AS place_count,
                (SELECT COUNT(*) FROM places p WHERE p.country_id = c.id AND p.status = 'been' AND p.deleted_at IS NULL) AS been_count,
                c.cover_relpath IS NOT NULL AS own_cover
         FROM countries c";

fn map_country(row: &rusqlite::Row<'_>) -> rusqlite::Result<Country> {
    Ok(Country {
        id: row.get(0)?,
        name: row.get(1)?,
        iso: row.get(2)?,
        cover_relpath: row.get(3)?,
        cover_x: row.get(4)?,
        cover_y: row.get(5)?,
        own_cover: row.get(10)?,
        sort: row.get(6)?,
        created_at: row.get(7)?,
        place_count: row.get(8)?,
        been_count: row.get(9)?,
    })
}

pub fn list_countries(conn: &Connection) -> Result<Vec<Country>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{COUNTRY_SELECT} WHERE c.deleted_at IS NULL ORDER BY c.sort ASC, c.name ASC"
    ))?;
    let rows = stmt.query_map([], map_country)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn create_country(conn: &Connection, name: &str, iso: Option<&str>) -> Result<Country, DbError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Country name is required"));
    }
    if country_exists_named(conn, trimmed)? {
        return Err(DbError::msg("That country is already on the wall"));
    }

    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM countries",
        [],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO countries (id, name, iso, cover_relpath, sort, created_at)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        params![id, trimmed, iso.map(str::trim).filter(|s| !s.is_empty()), sort, created_at],
    )?;

    Ok(Country {
        id,
        name: trimmed.to_string(),
        iso: iso.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string),
        cover_relpath: None,
        cover_x: 50.0,
        cover_y: 50.0,
        own_cover: false,
        sort,
        created_at,
        place_count: 0,
        been_count: 0,
    })
}

pub fn get_country(conn: &Connection, country_id: &str) -> Result<Country, DbError> {
    conn.query_row(
        &format!("{COUNTRY_SELECT} WHERE c.id = ?1 AND c.deleted_at IS NULL"),
        params![country_id],
        map_country,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Country not found"),
        other => DbError::from(other),
    })
}

pub fn update_country(
    conn: &Connection,
    country_id: &str,
    name: Option<&str>,
    iso: Option<&str>,
) -> Result<Country, DbError> {
    let mut country = get_country(conn, country_id)?;
    if let Some(name) = name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DbError::msg("Country name is required"));
        }
        country.name = trimmed.to_string();
    }
    if let Some(iso) = iso {
        let trimmed = iso.trim();
        country.iso = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    conn.execute(
        "UPDATE countries SET name = ?1, iso = ?2 WHERE id = ?3",
        params![country.name, country.iso, country_id],
    )?;
    get_country(conn, country_id)
}

pub fn delete_country(conn: &Connection, country_id: &str) -> Result<(), DbError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE images SET deleted_at = ?1 WHERE deleted_at IS NULL AND place_id IN (
            SELECT id FROM places WHERE country_id = ?2
         )",
        params![now, country_id],
    )?;
    conn.execute(
        "UPDATE places SET deleted_at = ?1 WHERE country_id = ?2 AND deleted_at IS NULL",
        params![now, country_id],
    )?;
    let changed = conn.execute(
        "UPDATE countries SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, country_id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Country not found"));
    }
    Ok(())
}

pub fn country_exists_named(conn: &Connection, name: &str) -> Result<bool, DbError> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM countries WHERE deleted_at IS NULL AND lower(name) = lower(?1)",
        params![name.trim()],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn ensure_place(
    conn: &Connection,
    country_id: &str,
    name: &str,
) -> Result<Place, DbError> {
    let trimmed = name.trim();
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM places WHERE country_id = ?1 AND deleted_at IS NULL AND lower(name) = lower(?2)",
            params![country_id, trimmed],
            |row| row.get(0),
        )
        .ok();
    if let Some(id) = existing {
        return get_place(conn, &id);
    }
    create_place(conn, country_id, trimmed, None)
}

pub fn reorder(conn: &Connection, table: &str, ids: &[String]) -> Result<(), DbError> {
    if table != "countries" && table != "places" {
        return Err(DbError::msg("Cannot reorder that list"));
    }
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            &format!("UPDATE {table} SET sort = ?1 WHERE id = ?2"),
            params![index as i32, id],
        )?;
    }
    Ok(())
}

fn map_place(row: &rusqlite::Row<'_>) -> rusqlite::Result<Place> {
    Ok(Place {
        id: row.get(0)?,
        country_id: row.get(1)?,
        name: row.get(2)?,
        cover_relpath: row.get(3)?,
        cover_x: row.get(9)?,
        cover_y: row.get(10)?,
        notes: row.get(4)?,
        status: row.get(5)?,
        sort: row.get(6)?,
        created_at: row.get(7)?,
        image_count: row.get(8)?,
        when_text: row.get(11)?,
        year: row.get(12)?,
    })
}

const PLACE_SELECT: &str = "SELECT id, country_id, name, cover_relpath, notes, status, sort, created_at,
        (SELECT COUNT(*) FROM images i WHERE i.place_id = places.id AND i.deleted_at IS NULL) AS image_count,
        cover_x, cover_y, when_text, year
 FROM places";

pub fn list_places(conn: &Connection, country_id: &str) -> Result<Vec<Place>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{PLACE_SELECT} WHERE country_id = ?1 AND deleted_at IS NULL ORDER BY sort ASC, name ASC"
    ))?;
    let rows = stmt.query_map(params![country_id], map_place)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn list_all_places(conn: &Connection) -> Result<Vec<Place>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{PLACE_SELECT} WHERE deleted_at IS NULL ORDER BY name ASC"
    ))?;
    let rows = stmt.query_map([], map_place)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn get_place(conn: &Connection, place_id: &str) -> Result<Place, DbError> {
    conn.query_row(
        &format!("{PLACE_SELECT} WHERE id = ?1 AND deleted_at IS NULL"),
        params![place_id],
        map_place,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Place not found"),
        other => DbError::from(other),
    })
}

pub fn create_place(
    conn: &Connection,
    country_id: &str,
    name: &str,
    status: Option<&str>,
) -> Result<Place, DbError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Place name is required"));
    }

    let exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM countries WHERE id = ?1 AND deleted_at IS NULL",
        params![country_id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(DbError::msg("Country not found"));
    }

    let status = match status.unwrap_or("dream") {
        "been" => "been",
        _ => "dream",
    };

    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM places WHERE country_id = ?1 AND deleted_at IS NULL",
        params![country_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO places (id, country_id, name, cover_relpath, notes, status, sort, created_at)
         VALUES (?1, ?2, ?3, NULL, NULL, ?4, ?5, ?6)",
        params![id, country_id, trimmed, status, sort, created_at],
    )?;

    Ok(Place {
        id,
        country_id: country_id.to_string(),
        name: trimmed.to_string(),
        cover_relpath: None,
        cover_x: 50.0,
        cover_y: 50.0,
        notes: None,
        status: status.to_string(),
        sort,
        created_at,
        image_count: 0,
        when_text: None,
        year: None,
    })
}

pub fn update_place(
    conn: &Connection,
    place_id: &str,
    name: Option<&str>,
    notes: Option<&str>,
    status: Option<&str>,
    when_text: Option<&str>,
) -> Result<Place, DbError> {
    let mut place = get_place(conn, place_id)?;

    if let Some(name) = name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DbError::msg("Place name is required"));
        }
        place.name = trimmed.to_string();
    }

    if let Some(notes) = notes {
        let trimmed = notes.trim();
        place.notes = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }

    if let Some(status) = status {
        place.status = match status {
            "been" => "been".to_string(),
            _ => "dream".to_string(),
        };
    }

    if let Some(when_text) = when_text {
        let trimmed = when_text.trim();
        if trimmed.is_empty() {
            place.when_text = None;
            place.year = None;
        } else {
            place.year = year_in(trimmed);
            place.when_text = Some(trimmed.to_string());
        }
    }

    conn.execute(
        "UPDATE places SET name = ?1, notes = ?2, status = ?3, when_text = ?4, year = ?5 WHERE id = ?6",
        params![place.name, place.notes, place.status, place.when_text, place.year, place_id],
    )?;

    get_place(conn, place_id)
}

pub fn delete_place(conn: &Connection, place_id: &str) -> Result<(), DbError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE images SET deleted_at = ?1 WHERE place_id = ?2 AND deleted_at IS NULL",
        params![now, place_id],
    )?;
    let changed = conn.execute(
        "UPDATE places SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, place_id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Place not found"));
    }
    Ok(())
}

fn map_image(row: &rusqlite::Row<'_>) -> rusqlite::Result<ImageRecord> {
    Ok(ImageRecord {
        id: row.get(0)?,
        place_id: row.get(1)?,
        original_relpath: row.get(2)?,
        display_relpath: row.get(3)?,
        thumb_relpath: row.get(4)?,
        sort: row.get(5)?,
        sha256: row.get(6)?,
        caption: row.get(7)?,
        taken_at: row.get(8)?,
    })
}

const IMAGE_SELECT: &str = "SELECT id, place_id, original_relpath, display_relpath, thumb_relpath, sort,
        sha256, caption, taken_at FROM images";

pub fn list_images(conn: &Connection, place_id: &str) -> Result<Vec<ImageRecord>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{IMAGE_SELECT} WHERE place_id = ?1 AND deleted_at IS NULL ORDER BY sort ASC"
    ))?;
    let rows = stmt.query_map(params![place_id], map_image)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn list_country_photos(
    conn: &Connection,
    country_id: &str,
    limit: i32,
) -> Result<Vec<ImageRecord>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{IMAGE_SELECT} WHERE deleted_at IS NULL AND place_id IN (
            SELECT id FROM places WHERE country_id = ?1 AND deleted_at IS NULL
         ) ORDER BY sort DESC LIMIT ?2"
    ))?;
    let rows = stmt.query_map(params![country_id, limit], map_image)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn find_image_by_hash(
    conn: &Connection,
    place_id: &str,
    sha256: &str,
) -> Result<Option<ImageRecord>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{IMAGE_SELECT} WHERE place_id = ?1 AND sha256 = ?2 AND deleted_at IS NULL LIMIT 1"
    ))?;
    let mut rows = stmt.query_map(params![place_id, sha256], map_image)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn insert_image(
    conn: &Connection,
    place_id: &str,
    original_relpath: &str,
    display_relpath: &str,
    thumb_relpath: &str,
    sha256: Option<&str>,
    caption: Option<&str>,
    taken_at: Option<&str>,
) -> Result<ImageRecord, DbError> {
    let id = Uuid::new_v4().to_string();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM images WHERE place_id = ?1 AND deleted_at IS NULL",
        params![place_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO images (id, place_id, original_relpath, display_relpath, thumb_relpath, sort, sha256, caption, taken_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            place_id,
            original_relpath,
            display_relpath,
            thumb_relpath,
            sort,
            sha256,
            caption,
            taken_at
        ],
    )?;

    let cover: Option<String> = conn
        .query_row(
            "SELECT cover_relpath FROM places WHERE id = ?1",
            params![place_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    if cover.is_none() {
        conn.execute(
            "UPDATE places SET cover_relpath = ?1 WHERE id = ?2",
            params![thumb_relpath, place_id],
        )?;
    }

    // First photo in a country also fills the country card when it has no cover yet.
    let place = get_place(conn, place_id)?;
    let country_cover: Option<String> = conn
        .query_row(
            "SELECT cover_relpath FROM countries WHERE id = ?1",
            params![place.country_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    if country_cover.is_none() {
        conn.execute(
            "UPDATE countries SET cover_relpath = ?1 WHERE id = ?2",
            params![thumb_relpath, place.country_id],
        )?;
    }

    Ok(ImageRecord {
        id,
        place_id: place_id.to_string(),
        original_relpath: original_relpath.to_string(),
        display_relpath: display_relpath.to_string(),
        thumb_relpath: thumb_relpath.to_string(),
        sort,
        sha256: sha256.map(str::to_string),
        caption: caption.map(str::to_string),
        taken_at: taken_at.map(str::to_string),
    })
}

pub fn get_image(conn: &Connection, image_id: &str) -> Result<ImageRecord, DbError> {
    conn.query_row(
        &format!("{IMAGE_SELECT} WHERE id = ?1"),
        params![image_id],
        map_image,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Image not found"),
        other => DbError::from(other),
    })
}

pub fn update_image_caption(
    conn: &Connection,
    image_id: &str,
    caption: Option<&str>,
) -> Result<ImageRecord, DbError> {
    let caption = caption.map(str::trim).filter(|s| !s.is_empty());
    conn.execute(
        "UPDATE images SET caption = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![caption, image_id],
    )?;
    get_image(conn, image_id)
}

pub fn delete_image(state: &AppState, image_id: &str) -> Result<(), DbError> {
    let image = get_image(&state.conn, image_id)?;
    let place = get_place(&state.conn, &image.place_id)?;
    let now = Utc::now().to_rfc3339();
    state.conn.execute(
        "UPDATE images SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, image_id],
    )?;

    if place.cover_relpath.as_deref() == Some(image.thumb_relpath.as_str())
        || place.cover_relpath.as_deref() == Some(image.display_relpath.as_str())
    {
        let next: Option<String> = state
            .conn
            .query_row(
                "SELECT thumb_relpath FROM images WHERE place_id = ?1 ORDER BY sort ASC LIMIT 1",
                params![image.place_id],
                |row| row.get(0),
            )
            .ok();
        state.conn.execute(
            "UPDATE places SET cover_relpath = ?1 WHERE id = ?2",
            params![next, image.place_id],
        )?;
    }

    Ok(())
}

pub fn set_place_cover(conn: &Connection, place_id: &str, image_id: &str) -> Result<Place, DbError> {
    let image = get_image(conn, image_id)?;
    if image.place_id != place_id {
        return Err(DbError::msg("Image does not belong to this place"));
    }
    conn.execute(
        "UPDATE places SET cover_relpath = ?1 WHERE id = ?2",
        params![image.thumb_relpath, place_id],
    )?;
    get_place(conn, place_id)
}

pub fn set_country_cover(
    conn: &Connection,
    country_id: &str,
    image_id: &str,
) -> Result<Country, DbError> {
    let image = get_image(conn, image_id)?;
    let place = get_place(conn, &image.place_id)?;
    if place.country_id != country_id {
        return Err(DbError::msg("Image does not belong to this country"));
    }
    conn.execute(
        "UPDATE countries SET cover_relpath = ?1 WHERE id = ?2",
        params![image.thumb_relpath, country_id],
    )?;
    get_country(conn, country_id)
}

pub fn relpath_from_abs(library_root: &Path, abs: &Path) -> Result<String, DbError> {
    abs.strip_prefix(library_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| DbError::msg("Path is outside library root"))
}

pub fn abs_from_relpath(library_root: &Path, relpath: &str) -> PathBuf {
    library_root.join(relpath.replace('/', std::path::MAIN_SEPARATOR_STR))
}

pub fn remove_relpath(state: &AppState, relpath: Option<&str>) {
    if let Some(rel) = relpath.filter(|s| !s.is_empty()) {
        let _ = fs::remove_file(abs_from_relpath(&state.library_root, rel));
    }
}

pub fn delete_country_with_files(state: &AppState, country_id: &str) -> Result<(), DbError> {
    delete_country(&state.conn, country_id)
}

pub fn get_link(conn: &Connection, id: &str) -> Result<LinkRecord, DbError> {
    conn.query_row(
        "SELECT id, place_id, country_id, url, title, thumb_relpath, source
         FROM links WHERE id = ?1",
        params![id],
        |row| {
            Ok(LinkRecord {
                id: row.get(0)?,
                place_id: row.get(1)?,
                country_id: row.get(2)?,
                url: row.get(3)?,
                title: row.get(4)?,
                thumb_relpath: row.get(5)?,
                source: row.get(6)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Link not found"),
        other => DbError::from(other),
    })
}

pub fn delete_link_with_files(state: &AppState, id: &str) -> Result<(), DbError> {
    let link = get_link(&state.conn, id)?;
    delete_link(&state.conn, id)?;
    remove_relpath(state, link.thumb_relpath.as_deref());
    Ok(())
}

pub fn sweep_orphans(state: &AppState) -> Result<u32, DbError> {
    let mut keep = std::collections::HashSet::new();

    let mut images = state.conn.prepare(
        "SELECT original_relpath, display_relpath, thumb_relpath FROM images",
    )?;
    let rows = images.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    for row in rows {
        let (a, b, c) = row?;
        keep.insert(a);
        keep.insert(b);
        keep.insert(c);
    }

    let mut covers = state.conn.prepare(
        "SELECT cover_relpath FROM countries WHERE cover_relpath IS NOT NULL
         UNION
         SELECT cover_relpath FROM places WHERE cover_relpath IS NOT NULL
         UNION
         SELECT cover_relpath FROM goals WHERE cover_relpath IS NOT NULL
         UNION
         SELECT cover_relpath FROM trips WHERE cover_relpath IS NOT NULL
         UNION
         SELECT thumb_relpath FROM inbox_items WHERE thumb_relpath IS NOT NULL
         UNION
         SELECT image_relpath FROM inbox_items WHERE image_relpath IS NOT NULL
         UNION
         SELECT thumb_relpath FROM links WHERE thumb_relpath IS NOT NULL",
    )?;
    let cover_rows = covers.query_map([], |row| row.get::<_, String>(0))?;
    for row in cover_rows {
        keep.insert(row?);
    }

    let library = state.library_root.join("library");
    if !library.exists() {
        return Ok(0);
    }

    let mut removed = 0u32;
    sweep_dir(state, &library, &keep, &mut removed)?;
    Ok(removed)
}

fn sweep_dir(
    state: &AppState,
    current: &Path,
    keep: &std::collections::HashSet<String>,
    removed: &mut u32,
) -> Result<(), DbError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            sweep_dir(state, &path, keep, removed)?;
            if fs::read_dir(&path).ok().and_then(|mut d| d.next()).is_none() {
                let _ = fs::remove_dir(&path);
            }
        } else if let Ok(rel) = relpath_from_abs(&state.library_root, &path) {
            if !keep.contains(&rel) {
                if fs::remove_file(&path).is_ok() {
                    *removed += 1;
                }
            }
        }
    }
    Ok(())
}

pub fn link_url_exists(conn: &Connection, url: &str) -> Result<bool, DbError> {
    let exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM links WHERE url = ?1",
        params![url],
        |row| row.get(0),
    )?;
    Ok(exists > 0)
}

pub fn set_link_preview(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    thumb_relpath: Option<&str>,
) -> Result<(), DbError> {
    if let Some(title) = title {
        conn.execute(
            "UPDATE links SET title = ?1 WHERE id = ?2 AND title IS NULL",
            params![title, id],
        )?;
    }
    if let Some(rel) = thumb_relpath {
        conn.execute(
            "UPDATE links SET thumb_relpath = ?1 WHERE id = ?2 AND thumb_relpath IS NULL",
            params![rel, id],
        )?;
    }
    Ok(())
}

pub fn insert_link(
    conn: &Connection,
    url: &str,
    title: Option<&str>,
    thumb_relpath: Option<&str>,
    source: &str,
    place_id: Option<&str>,
    country_id: Option<&str>,
) -> Result<LinkRecord, DbError> {
    let exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM links WHERE url = ?1",
        params![url],
        |row| row.get(0),
    )?;
    if exists > 0 {
        return Err(DbError::msg("That link is already saved"));
    }
    let id = Uuid::new_v4().to_string();
    let source = if source == "youtube" { "youtube" } else { "web" };
    conn.execute(
        "INSERT INTO links (id, place_id, country_id, url, title, thumb_relpath, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, place_id, country_id, url, title, thumb_relpath, source],
    )?;
    Ok(LinkRecord {
        id,
        place_id: place_id.map(str::to_string),
        country_id: country_id.map(str::to_string),
        url: url.to_string(),
        title: title.map(str::to_string),
        thumb_relpath: thumb_relpath.map(str::to_string),
        source: source.to_string(),
    })
}

pub fn list_links(
    conn: &Connection,
    place_id: Option<&str>,
    country_id: Option<&str>,
) -> Result<Vec<LinkRecord>, DbError> {
    let mut stmt = if place_id.is_some() {
        conn.prepare(
            "SELECT id, place_id, country_id, url, title, thumb_relpath, source
             FROM links WHERE place_id = ?1 ORDER BY title ASC",
        )?
    } else {
        conn.prepare(
            "SELECT id, place_id, country_id, url, title, thumb_relpath, source
             FROM links WHERE country_id = ?1 AND place_id IS NULL ORDER BY title ASC",
        )?
    };
    let key = place_id.or(country_id).ok_or_else(|| DbError::msg("Need a place or country"))?;
    let rows = stmt.query_map(params![key], |row| {
        Ok(LinkRecord {
            id: row.get(0)?,
            place_id: row.get(1)?,
            country_id: row.get(2)?,
            url: row.get(3)?,
            title: row.get(4)?,
            thumb_relpath: row.get(5)?,
            source: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn delete_link(conn: &Connection, id: &str) -> Result<(), DbError> {
    let changed = conn.execute("DELETE FROM links WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::msg("Link not found"));
    }
    Ok(())
}

const GOAL_SELECT: &str = "SELECT g.id, g.name, g.notes, g.status, g.cover_relpath,
            c.credit, c.credit_url, g.sort, g.created_at, g.cover_x, g.cover_y
     FROM goals g
     LEFT JOIN cover_credits c ON c.relpath = g.cover_relpath";

fn map_goal(row: &rusqlite::Row<'_>) -> rusqlite::Result<Goal> {
    Ok(Goal {
        id: row.get(0)?,
        name: row.get(1)?,
        notes: row.get(2)?,
        status: row.get(3)?,
        cover_relpath: row.get(4)?,
        cover_credit: row.get(5)?,
        cover_credit_url: row.get(6)?,
        cover_x: row.get(9)?,
        cover_y: row.get(10)?,
        sort: row.get(7)?,
        created_at: row.get(8)?,
    })
}

pub fn list_goals(conn: &Connection) -> Result<Vec<Goal>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{GOAL_SELECT} WHERE g.deleted_at IS NULL ORDER BY g.sort ASC, g.name ASC"
    ))?;
    let rows = stmt.query_map([], map_goal)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn get_goal(conn: &Connection, goal_id: &str) -> Result<Goal, DbError> {
    conn.query_row(
        &format!("{GOAL_SELECT} WHERE g.id = ?1 AND g.deleted_at IS NULL"),
        params![goal_id],
        map_goal,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Goal not found"),
        other => DbError::from(other),
    })
}

pub fn create_goal(conn: &Connection, name: &str) -> Result<Goal, DbError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Name is required"));
    }
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM goals WHERE deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO goals (id, name, notes, status, cover_relpath, sort, created_at)
         VALUES (?1, ?2, NULL, 'dream', NULL, ?3, ?4)",
        params![id, trimmed, sort, created_at],
    )?;
    get_goal(conn, &id)
}

pub fn update_goal(
    conn: &Connection,
    goal_id: &str,
    name: Option<&str>,
    notes: Option<&str>,
    status: Option<&str>,
) -> Result<Goal, DbError> {
    let mut goal = get_goal(conn, goal_id)?;
    if let Some(name) = name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DbError::msg("Name is required"));
        }
        goal.name = trimmed.to_string();
    }
    if let Some(notes) = notes {
        let trimmed = notes.trim();
        goal.notes = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Some(status) = status {
        goal.status = match status {
            "done" | "been" => "done".to_string(),
            _ => "dream".to_string(),
        };
    }
    conn.execute(
        "UPDATE goals SET name = ?1, notes = ?2, status = ?3 WHERE id = ?4",
        params![goal.name, goal.notes, goal.status, goal_id],
    )?;
    get_goal(conn, goal_id)
}

pub fn delete_goal(conn: &Connection, goal_id: &str) -> Result<(), DbError> {
    let now = Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE goals SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, goal_id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Goal not found"));
    }
    Ok(())
}

pub fn reorder_goals(conn: &Connection, ids: &[String]) -> Result<(), DbError> {
    for (index, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE goals SET sort = ?1 WHERE id = ?2",
            params![index as i32, id],
        )?;
    }
    Ok(())
}

pub fn cover_credit(conn: &Connection, relpath: &str) -> Result<Option<(String, Option<String>)>, DbError> {
    let found = conn.query_row(
        "SELECT credit, credit_url FROM cover_credits WHERE relpath = ?1",
        params![relpath],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
    );
    match found {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(other) => Err(DbError::from(other)),
    }
}

fn cover_table(target: &str) -> Result<(&'static str, &'static str), DbError> {
    match target {
        "goal" => Ok(("goals", "AND deleted_at IS NULL")),
        "trip" => Ok(("trips", "AND deleted_at IS NULL")),
        "country" => Ok(("countries", "AND deleted_at IS NULL")),
        "place" => Ok(("places", "AND deleted_at IS NULL")),
        _ => Err(DbError::msg("Unknown cover target")),
    }
}

fn focus_axis(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        50.0
    }
}

pub fn assign_cover(
    conn: &Connection,
    target: &str,
    id: &str,
    relpath: &str,
    credit: Option<&str>,
    credit_url: Option<&str>,
    focus_x: f64,
    focus_y: f64,
) -> Result<Option<String>, DbError> {
    let (table, extra) = cover_table(target)?;
    let old: Option<String> = conn
        .query_row(
            &format!("SELECT cover_relpath FROM {table} WHERE id = ?1 {extra}"),
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::msg("Not found"),
            other => DbError::from(other),
        })?;
    let changed = conn.execute(
        &format!("UPDATE {table} SET cover_relpath = ?1, cover_x = ?2, cover_y = ?3 WHERE id = ?4 {extra}"),
        params![relpath, focus_axis(focus_x), focus_axis(focus_y), id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Not found"));
    }
    if let Some(credit) = credit.map(str::trim).filter(|s| !s.is_empty()) {
        conn.execute(
            "INSERT INTO cover_credits (relpath, credit, credit_url) VALUES (?1, ?2, ?3)
             ON CONFLICT(relpath) DO UPDATE SET credit = excluded.credit, credit_url = excluded.credit_url",
            params![relpath, credit, credit_url.map(str::trim).filter(|s| !s.is_empty())],
        )?;
    }
    Ok(old)
}

pub fn set_cover_focus(
    conn: &Connection,
    target: &str,
    id: &str,
    focus_x: f64,
    focus_y: f64,
) -> Result<(), DbError> {
    let (table, extra) = cover_table(target)?;
    let changed = conn.execute(
        &format!("UPDATE {table} SET cover_x = ?1, cover_y = ?2 WHERE id = ?3 {extra}"),
        params![focus_axis(focus_x), focus_axis(focus_y), id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Not found"));
    }
    Ok(())
}

pub fn retire_unreferenced_cover(state: &AppState, relpath: Option<&str>) {
    let Some(relpath) = relpath.filter(|s| s.starts_with("library/covers/")) else {
        return;
    };
    let still: i32 = state
        .conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM goals WHERE cover_relpath = ?1) +
                (SELECT COUNT(*) FROM trips WHERE cover_relpath = ?1) +
                (SELECT COUNT(*) FROM countries WHERE cover_relpath = ?1) +
                (SELECT COUNT(*) FROM places WHERE cover_relpath = ?1)",
            params![relpath],
            |row| row.get(0),
        )
        .unwrap_or(1);
    if still == 0 {
        remove_relpath(state, Some(relpath));
        let _ = state
            .conn
            .execute("DELETE FROM cover_credits WHERE relpath = ?1", params![relpath]);
    }
}

const TRIP_COSTS: &[&str] = &[
    "visa",
    "sim",
    "flights",
    "hotels",
    "food",
    "transport",
    "tickets",
    "activities",
    "guide",
    "insurance",
    "other",
];

fn year_in(text: &str) -> Option<i32> {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index + 3 < chars.len() {
        let slice = &chars[index..index + 4];
        let digits = slice.iter().all(|c| c.is_ascii_digit());
        let before = index == 0 || !chars[index - 1].is_ascii_digit();
        let after = index + 4 == chars.len() || !chars[index + 4].is_ascii_digit();
        if digits && before && after {
            let year: i32 = slice.iter().collect::<String>().parse().ok()?;
            if (1900..2200).contains(&year) {
                return Some(year);
            }
        }
        index += 1;
    }
    None
}

fn clean_currency(raw: &str) -> Result<Option<String>, DbError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 8 {
        return Err(DbError::msg("Currency should be a short label, like INR or USD"));
    }
    Ok(Some(trimmed.to_string()))
}

const TRIP_SELECT: &str = "SELECT t.id, t.name, t.when_text, t.year, t.currency, t.notes, t.status,
        t.cover_relpath, c.credit, c.credit_url, t.cover_x, t.cover_y, t.sort, t.created_at,
        (SELECT COUNT(*) FROM trip_stops s
            JOIN places p ON p.id = s.place_id AND p.deleted_at IS NULL
            WHERE s.trip_id = t.id) AS stop_count,
        (SELECT COALESCE(SUM(amount), 0) FROM trip_costs k WHERE k.trip_id = t.id) AS total,
        (SELECT COUNT(*) FROM trip_costs k WHERE k.trip_id = t.id) AS cost_count
     FROM trips t
     LEFT JOIN cover_credits c ON c.relpath = t.cover_relpath";

fn map_trip(row: &rusqlite::Row<'_>) -> rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        when_text: row.get(2)?,
        year: row.get(3)?,
        currency: row.get(4)?,
        notes: row.get(5)?,
        status: row.get(6)?,
        cover_relpath: row.get(7)?,
        cover_credit: row.get(8)?,
        cover_credit_url: row.get(9)?,
        cover_x: row.get(10)?,
        cover_y: row.get(11)?,
        sort: row.get(12)?,
        created_at: row.get(13)?,
        stop_count: row.get(14)?,
        total: row.get(15)?,
        cost_count: row.get(16)?,
    })
}

pub fn list_trips(conn: &Connection) -> Result<Vec<Trip>, DbError> {
    let mut stmt = conn.prepare(&format!(
        "{TRIP_SELECT} WHERE t.deleted_at IS NULL ORDER BY t.sort ASC, t.name ASC"
    ))?;
    let rows = stmt.query_map([], map_trip)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn get_trip(conn: &Connection, trip_id: &str) -> Result<Trip, DbError> {
    conn.query_row(
        &format!("{TRIP_SELECT} WHERE t.id = ?1 AND t.deleted_at IS NULL"),
        params![trip_id],
        map_trip,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => DbError::msg("Trip not found"),
        other => DbError::from(other),
    })
}

pub fn create_trip(conn: &Connection, name: &str) -> Result<Trip, DbError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Name is required"));
    }
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM trips WHERE deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO trips (id, name, status, sort, created_at) VALUES (?1, ?2, 'dream', ?3, ?4)",
        params![id, trimmed, sort, created_at],
    )?;
    get_trip(conn, &id)
}

pub fn update_trip(
    conn: &Connection,
    trip_id: &str,
    name: Option<&str>,
    when_text: Option<&str>,
    currency: Option<&str>,
    notes: Option<&str>,
    status: Option<&str>,
    mark_places: bool,
) -> Result<Trip, DbError> {
    let mut trip = get_trip(conn, trip_id)?;
    if let Some(name) = name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DbError::msg("Name is required"));
        }
        trip.name = trimmed.to_string();
    }
    if let Some(when_text) = when_text {
        let trimmed = when_text.trim();
        if trimmed.is_empty() {
            trip.when_text = None;
            trip.year = None;
        } else {
            trip.year = year_in(trimmed);
            trip.when_text = Some(trimmed.to_string());
        }
    }
    if let Some(currency) = currency {
        trip.currency = clean_currency(currency)?;
    }
    if let Some(notes) = notes {
        let trimmed = notes.trim();
        trip.notes = if trimmed.is_empty() { None } else { Some(trimmed.to_string()) };
    }
    let marking_done = status.is_some_and(|value| value == "done" || value == "been") && trip.status != "done";
    if let Some(status) = status {
        trip.status = match status {
            "done" | "been" => "done".to_string(),
            _ => "dream".to_string(),
        };
    }
    conn.execute(
        "UPDATE trips SET name = ?1, when_text = ?2, year = ?3, currency = ?4, notes = ?5, status = ?6 WHERE id = ?7",
        params![trip.name, trip.when_text, trip.year, trip.currency, trip.notes, trip.status, trip_id],
    )?;
    if marking_done && mark_places {
        conn.execute(
            "UPDATE places SET status = 'been'
             WHERE deleted_at IS NULL AND id IN (SELECT place_id FROM trip_stops WHERE trip_id = ?1)",
            params![trip_id],
        )?;
    }
    get_trip(conn, trip_id)
}

pub fn delete_trip(conn: &Connection, trip_id: &str) -> Result<(), DbError> {
    let now = Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE trips SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, trip_id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("Trip not found"));
    }
    Ok(())
}

pub fn list_trip_stops(conn: &Connection, trip_id: &str) -> Result<Vec<TripStop>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT s.place_id, p.country_id, p.name, c.name, p.status, s.sort
         FROM trip_stops s
         JOIN places p ON p.id = s.place_id AND p.deleted_at IS NULL
         JOIN countries c ON c.id = p.country_id
         WHERE s.trip_id = ?1
         ORDER BY s.sort ASC, p.name ASC",
    )?;
    let rows = stmt.query_map(params![trip_id], |row| {
        Ok(TripStop {
            place_id: row.get(0)?,
            country_id: row.get(1)?,
            place_name: row.get(2)?,
            country_name: row.get(3)?,
            status: row.get(4)?,
            sort: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn add_trip_stop(conn: &Connection, trip_id: &str, place_id: &str) -> Result<(), DbError> {
    get_trip(conn, trip_id)?;
    get_place(conn, place_id)?;
    let exists: i32 = conn.query_row(
        "SELECT COUNT(*) FROM trip_stops WHERE trip_id = ?1 AND place_id = ?2",
        params![trip_id, place_id],
        |row| row.get(0),
    )?;
    if exists > 0 {
        return Err(DbError::msg("That place is already on this trip"));
    }
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM trip_stops WHERE trip_id = ?1",
        params![trip_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO trip_stops (trip_id, place_id, sort) VALUES (?1, ?2, ?3)",
        params![trip_id, place_id, sort],
    )?;
    Ok(())
}

pub fn remove_trip_stop(conn: &Connection, trip_id: &str, place_id: &str) -> Result<(), DbError> {
    let changed = conn.execute(
        "DELETE FROM trip_stops WHERE trip_id = ?1 AND place_id = ?2",
        params![trip_id, place_id],
    )?;
    if changed == 0 {
        return Err(DbError::msg("That place is not on this trip"));
    }
    Ok(())
}

pub fn reorder_trip_stops(conn: &Connection, trip_id: &str, place_ids: &[String]) -> Result<(), DbError> {
    get_trip(conn, trip_id)?;
    for (index, place_id) in place_ids.iter().enumerate() {
        let changed = conn.execute(
            "UPDATE trip_stops SET sort = ?1 WHERE trip_id = ?2 AND place_id = ?3",
            params![index as i32, trip_id, place_id],
        )?;
        if changed == 0 {
            return Err(DbError::msg("That place is not on this trip"));
        }
    }
    Ok(())
}

pub fn list_trip_costs(conn: &Connection, trip_id: &str) -> Result<Vec<TripCost>, DbError> {
    let mut stmt = conn.prepare("SELECT category, amount FROM trip_costs WHERE trip_id = ?1")?;
    let rows = stmt.query_map(params![trip_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let stored = rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)?;
    let mut costs = Vec::with_capacity(TRIP_COSTS.len());
    for key in TRIP_COSTS {
        let amount = stored.iter().find(|(category, _)| category == key).map(|(_, amount)| *amount);
        costs.push(TripCost {
            category: (*key).to_string(),
            amount,
        });
    }
    for (category, amount) in stored {
        if !TRIP_COSTS.contains(&category.as_str()) {
            costs.push(TripCost {
                category,
                amount: Some(amount),
            });
        }
    }
    Ok(costs)
}

pub fn trip_detail(conn: &Connection, trip_id: &str) -> Result<TripDetail, DbError> {
    Ok(TripDetail {
        trip: get_trip(conn, trip_id)?,
        stops: list_trip_stops(conn, trip_id)?,
        costs: list_trip_costs(conn, trip_id)?,
    })
}

pub fn set_trip_cost(
    conn: &Connection,
    trip_id: &str,
    category: &str,
    amount: Option<i64>,
) -> Result<TripDetail, DbError> {
    get_trip(conn, trip_id)?;
    if !TRIP_COSTS.contains(&category) {
        return Err(DbError::msg("Unknown expense"));
    }
    match amount {
        None => {
            conn.execute(
                "DELETE FROM trip_costs WHERE trip_id = ?1 AND category = ?2",
                params![trip_id, category],
            )?;
        }
        Some(amount) if (0..=1_000_000_000).contains(&amount) => {
            conn.execute(
                "INSERT INTO trip_costs (trip_id, category, amount) VALUES (?1, ?2, ?3)
                 ON CONFLICT(trip_id, category) DO UPDATE SET amount = excluded.amount",
                params![trip_id, category, amount],
            )?;
        }
        Some(_) => return Err(DbError::msg("Enter a whole amount from 0 up")),
    }
    trip_detail(conn, trip_id)
}

pub fn list_place_tasks(conn: &Connection, place_id: &str) -> Result<Vec<PlaceTask>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, place_id, body, done, sort FROM place_tasks WHERE place_id = ?1 ORDER BY sort ASC, body ASC",
    )?;
    let rows = stmt.query_map(params![place_id], |row| {
        Ok(PlaceTask {
            id: row.get(0)?,
            place_id: row.get(1)?,
            body: row.get(2)?,
            done: row.get::<_, i32>(3)? != 0,
            sort: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn add_place_task(conn: &Connection, place_id: &str, body: &str) -> Result<PlaceTask, DbError> {
    get_place(conn, place_id)?;
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Write the thing to do"));
    }
    if trimmed.chars().count() > 160 {
        return Err(DbError::msg("Keep that to a short line"));
    }
    let id = Uuid::new_v4().to_string();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM place_tasks WHERE place_id = ?1",
        params![place_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO place_tasks (id, place_id, body, done, sort) VALUES (?1, ?2, ?3, 0, ?4)",
        params![id, place_id, trimmed, sort],
    )?;
    Ok(PlaceTask {
        id,
        place_id: place_id.to_string(),
        body: trimmed.to_string(),
        done: false,
        sort,
    })
}

pub fn set_place_task(
    conn: &Connection,
    task_id: &str,
    body: Option<&str>,
    done: Option<bool>,
) -> Result<PlaceTask, DbError> {
    let mut task = conn
        .query_row(
            "SELECT id, place_id, body, done, sort FROM place_tasks WHERE id = ?1",
            params![task_id],
            |row| {
                Ok(PlaceTask {
                    id: row.get(0)?,
                    place_id: row.get(1)?,
                    body: row.get(2)?,
                    done: row.get::<_, i32>(3)? != 0,
                    sort: row.get(4)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::msg("Task not found"),
            other => DbError::from(other),
        })?;
    if let Some(body) = body {
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return Err(DbError::msg("Write the thing to do"));
        }
        task.body = trimmed.to_string();
    }
    if let Some(done) = done {
        task.done = done;
    }
    conn.execute(
        "UPDATE place_tasks SET body = ?1, done = ?2 WHERE id = ?3",
        params![task.body, i32::from(task.done), task_id],
    )?;
    Ok(task)
}

pub fn delete_place_task(conn: &Connection, task_id: &str) -> Result<(), DbError> {
    let changed = conn.execute("DELETE FROM place_tasks WHERE id = ?1", params![task_id])?;
    if changed == 0 {
        return Err(DbError::msg("Task not found"));
    }
    Ok(())
}

pub fn search(conn: &Connection, query: &str) -> Result<Vec<SearchHit>, DbError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let like = like_contains(needle);
    let mut hits = Vec::new();

    let mut countries = conn.prepare(
        "SELECT id, name, iso FROM countries WHERE deleted_at IS NULL AND (name LIKE ?1 ESCAPE '\\' OR IFNULL(iso, '') LIKE ?1 ESCAPE '\\') LIMIT 20",
    )?;
    for row in countries.query_map(params![like], |row| {
        Ok(SearchHit {
            kind: "country".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get::<_, Option<String>>(2)?,
            country_id: row.get(0)?,
        })
    })? {
        hits.push(row?);
    }

    let mut goals = conn.prepare(
        "SELECT id, name, notes FROM goals
         WHERE deleted_at IS NULL AND (name LIKE ?1 ESCAPE '\\' OR IFNULL(notes, '') LIKE ?1 ESCAPE '\\') LIMIT 20",
    )?;
    for row in goals.query_map(params![like], |row| {
        Ok(SearchHit {
            kind: "goal".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get(2)?,
            country_id: None,
        })
    })? {
        hits.push(row?);
    }

    let mut trips = conn.prepare(
        "SELECT id, name, when_text FROM trips
         WHERE deleted_at IS NULL AND (name LIKE ?1 ESCAPE '\\' OR IFNULL(when_text, '') LIKE ?1 ESCAPE '\\' OR IFNULL(notes, '') LIKE ?1 ESCAPE '\\') LIMIT 20",
    )?;
    for row in trips.query_map(params![like], |row| {
        Ok(SearchHit {
            kind: "trip".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get(2)?,
            country_id: None,
        })
    })? {
        hits.push(row?);
    }

    let mut places = conn.prepare(
        "SELECT id, name, notes, country_id FROM places
         WHERE deleted_at IS NULL AND (name LIKE ?1 ESCAPE '\\' OR IFNULL(notes, '') LIKE ?1 ESCAPE '\\') LIMIT 20",
    )?;
    for row in places.query_map(params![like], |row| {
        Ok(SearchHit {
            kind: "place".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get(2)?,
            country_id: row.get(3)?,
        })
    })? {
        hits.push(row?);
    }

    let mut links = conn.prepare(
        "SELECT id, IFNULL(title, url), url, country_id FROM links
         WHERE IFNULL(title, '') LIKE ?1 ESCAPE '\\' OR url LIKE ?1 ESCAPE '\\' LIMIT 20",
    )?;
    for row in links.query_map(params![like], |row| {
        Ok(SearchHit {
            kind: "link".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get(2)?,
            country_id: row.get(3)?,
        })
    })? {
        hits.push(row?);
    }

    Ok(hits)
}

pub fn list_trash(conn: &Connection) -> Result<Vec<TrashItem>, DbError> {
    let mut items = Vec::new();
    let mut countries = conn.prepare(
        "SELECT id, name, deleted_at FROM countries WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    )?;
    for row in countries.query_map([], |row| {
        Ok(TrashItem {
            kind: "country".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            deleted_at: row.get(2)?,
        })
    })? {
        items.push(row?);
    }
    let mut goals = conn.prepare(
        "SELECT id, name, deleted_at FROM goals WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    )?;
    for row in goals.query_map([], |row| {
        Ok(TrashItem {
            kind: "goal".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            deleted_at: row.get(2)?,
        })
    })? {
        items.push(row?);
    }
    let mut trips = conn.prepare(
        "SELECT id, name, deleted_at FROM trips WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    )?;
    for row in trips.query_map([], |row| {
        Ok(TrashItem {
            kind: "trip".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            deleted_at: row.get(2)?,
        })
    })? {
        items.push(row?);
    }
    let mut places = conn.prepare(
        "SELECT id, name, deleted_at FROM places WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    )?;
    for row in places.query_map([], |row| {
        Ok(TrashItem {
            kind: "place".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            deleted_at: row.get(2)?,
        })
    })? {
        items.push(row?);
    }
    let mut images = conn.prepare(
        "SELECT id, IFNULL(caption, original_relpath), deleted_at FROM images WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    )?;
    for row in images.query_map([], |row| {
        Ok(TrashItem {
            kind: "image".into(),
            id: row.get(0)?,
            title: row.get(1)?,
            deleted_at: row.get(2)?,
        })
    })? {
        items.push(row?);
    }
    items.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
    Ok(items)
}

pub fn restore_trash(conn: &Connection, kind: &str, id: &str) -> Result<(), DbError> {
    let changed = match kind {
        "country" => {
            conn.execute(
                "UPDATE images SET deleted_at = NULL WHERE place_id IN (SELECT id FROM places WHERE country_id = ?1)",
                params![id],
            )?;
            conn.execute(
                "UPDATE places SET deleted_at = NULL WHERE country_id = ?1",
                params![id],
            )?;
            conn.execute(
                "UPDATE countries SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )?
        }
        "place" => {
            conn.execute(
                "UPDATE images SET deleted_at = NULL WHERE place_id = ?1",
                params![id],
            )?;
            conn.execute(
                "UPDATE places SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )?
        }
        "image" => conn.execute(
            "UPDATE images SET deleted_at = NULL WHERE id = ?1",
            params![id],
        )?,
        "goal" => conn.execute(
            "UPDATE goals SET deleted_at = NULL WHERE id = ?1",
            params![id],
        )?,
        "trip" => conn.execute(
            "UPDATE trips SET deleted_at = NULL WHERE id = ?1",
            params![id],
        )?,
        _ => return Err(DbError::msg("Unknown trash item")),
    };
    if changed == 0 {
        return Err(DbError::msg("Trash item not found"));
    }
    Ok(())
}

pub fn purge_trash(state: &AppState, older_than_days: i64) -> Result<u32, DbError> {
    let cutoff = (Utc::now() - chrono::Duration::days(older_than_days)).to_rfc3339();
    let mut removed = 0u32;

    let mut images = state.conn.prepare(
        "SELECT id, original_relpath, display_relpath, thumb_relpath FROM images WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
    )?;
    let rows = images.query_map(params![cutoff], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let doomed: Vec<_> = rows.collect::<Result<Vec<_>, _>>()?;
    drop(images);
    for (id, a, b, c) in doomed {
        remove_relpath(state, Some(&a));
        remove_relpath(state, Some(&b));
        remove_relpath(state, Some(&c));
        state.conn.execute("DELETE FROM images WHERE id = ?1", params![id])?;
        removed += 1;
    }

    removed += state.conn.execute(
        "DELETE FROM places WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        params![cutoff],
    )? as u32;
    removed += state.conn.execute(
        "DELETE FROM countries WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        params![cutoff],
    )? as u32;

    let mut goals = state.conn.prepare(
        "SELECT id, cover_relpath FROM goals WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
    )?;
    let goal_rows = goals.query_map(params![cutoff], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let doomed_goals: Vec<_> = goal_rows.collect::<Result<Vec<_>, _>>()?;
    drop(goals);
    for (id, cover) in doomed_goals {
        if let Some(rel) = cover.filter(|s| s.starts_with("library/covers/")) {
            remove_relpath(state, Some(&rel));
            state
                .conn
                .execute("DELETE FROM cover_credits WHERE relpath = ?1", params![rel])?;
        }
        state.conn.execute("DELETE FROM goals WHERE id = ?1", params![id])?;
        removed += 1;
    }

    let mut trips = state.conn.prepare(
        "SELECT id, cover_relpath FROM trips WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
    )?;
    let trip_rows = trips.query_map(params![cutoff], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let doomed_trips: Vec<_> = trip_rows.collect::<Result<Vec<_>, _>>()?;
    drop(trips);
    for (id, cover) in doomed_trips {
        if let Some(rel) = cover.filter(|s| s.starts_with("library/covers/")) {
            remove_relpath(state, Some(&rel));
            state
                .conn
                .execute("DELETE FROM cover_credits WHERE relpath = ?1", params![rel])?;
        }
        state.conn.execute("DELETE FROM trip_stops WHERE trip_id = ?1", params![id])?;
        state.conn.execute("DELETE FROM trip_costs WHERE trip_id = ?1", params![id])?;
        state.conn.execute("DELETE FROM trips WHERE id = ?1", params![id])?;
        removed += 1;
    }
    Ok(removed)
}

fn like_contains(needle: &str) -> String {
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}
