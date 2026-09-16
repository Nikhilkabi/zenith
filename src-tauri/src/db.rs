use crate::models::{Country, ImageRecord, Place};
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

const MIGRATIONS: &[(i32, &str)] = &[(1, include_str!("migrations/001_initial.sql"))];

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

pub fn list_countries(conn: &Connection) -> Result<Vec<Country>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, iso, cover_relpath, sort, created_at
         FROM countries ORDER BY sort ASC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Country {
            id: row.get(0)?,
            name: row.get(1)?,
            iso: row.get(2)?,
            cover_relpath: row.get(3)?,
            sort: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn create_country(conn: &Connection, name: &str, iso: Option<&str>) -> Result<Country, DbError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Country name is required"));
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
        sort,
        created_at,
    })
}

pub fn list_places(conn: &Connection, country_id: &str) -> Result<Vec<Place>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, country_id, name, cover_relpath, notes, status, sort, created_at
         FROM places WHERE country_id = ?1 ORDER BY sort ASC, name ASC",
    )?;
    let rows = stmt.query_map(params![country_id], |row| {
        Ok(Place {
            id: row.get(0)?,
            country_id: row.get(1)?,
            name: row.get(2)?,
            cover_relpath: row.get(3)?,
            notes: row.get(4)?,
            status: row.get(5)?,
            sort: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn get_place(conn: &Connection, place_id: &str) -> Result<Place, DbError> {
    conn.query_row(
        "SELECT id, country_id, name, cover_relpath, notes, status, sort, created_at
         FROM places WHERE id = ?1",
        params![place_id],
        |row| {
            Ok(Place {
                id: row.get(0)?,
                country_id: row.get(1)?,
                name: row.get(2)?,
                cover_relpath: row.get(3)?,
                notes: row.get(4)?,
                status: row.get(5)?,
                sort: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
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
        "SELECT COUNT(*) FROM countries WHERE id = ?1",
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
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM places WHERE country_id = ?1",
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
        notes: None,
        status: status.to_string(),
        sort,
        created_at,
    })
}

pub fn update_place(
    conn: &Connection,
    place_id: &str,
    name: Option<&str>,
    notes: Option<&str>,
    status: Option<&str>,
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

    conn.execute(
        "UPDATE places SET name = ?1, notes = ?2, status = ?3 WHERE id = ?4",
        params![place.name, place.notes, place.status, place_id],
    )?;

    get_place(conn, place_id)
}

pub fn delete_place(conn: &Connection, place_id: &str) -> Result<(), DbError> {
    let changed = conn.execute("DELETE FROM places WHERE id = ?1", params![place_id])?;
    if changed == 0 {
        return Err(DbError::msg("Place not found"));
    }
    Ok(())
}

pub fn list_images(conn: &Connection, place_id: &str) -> Result<Vec<ImageRecord>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, place_id, original_relpath, display_relpath, thumb_relpath, sort
         FROM images WHERE place_id = ?1 ORDER BY sort ASC",
    )?;
    let rows = stmt.query_map(params![place_id], |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            place_id: row.get(1)?,
            original_relpath: row.get(2)?,
            display_relpath: row.get(3)?,
            thumb_relpath: row.get(4)?,
            sort: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn insert_image(
    conn: &Connection,
    place_id: &str,
    original_relpath: &str,
    display_relpath: &str,
    thumb_relpath: &str,
) -> Result<ImageRecord, DbError> {
    let id = Uuid::new_v4().to_string();
    let sort: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort), -1) + 1 FROM images WHERE place_id = ?1",
        params![place_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "INSERT INTO images (id, place_id, original_relpath, display_relpath, thumb_relpath, sort)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, place_id, original_relpath, display_relpath, thumb_relpath, sort],
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

    Ok(ImageRecord {
        id,
        place_id: place_id.to_string(),
        original_relpath: original_relpath.to_string(),
        display_relpath: display_relpath.to_string(),
        thumb_relpath: thumb_relpath.to_string(),
        sort,
    })
}

pub fn relpath_from_abs(library_root: &Path, abs: &Path) -> Result<String, DbError> {
    abs.strip_prefix(library_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| DbError::msg("Path is outside library root"))
}
