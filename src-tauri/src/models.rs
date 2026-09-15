use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub iso: Option<String>,
    pub cover_relpath: Option<String>,
    pub sort: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub country_id: String,
    pub name: String,
    pub cover_relpath: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub sort: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: String,
    pub place_id: String,
    pub original_relpath: String,
    pub display_relpath: String,
    pub thumb_relpath: String,
    pub sort: i32,
}
