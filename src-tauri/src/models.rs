use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub iso: Option<String>,
    pub cover_relpath: Option<String>,
    pub sort: i32,
    pub created_at: String,
    pub place_count: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRecord {
    pub id: String,
    pub place_id: Option<String>,
    pub country_id: Option<String>,
    pub url: String,
    pub title: Option<String>,
    pub thumb_relpath: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub thumb_relpath: Option<String>,
    pub image_relpath: Option<String>,
    pub source: String,
    pub created_at: String,
    pub filed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub country_id: Option<String>,
}
