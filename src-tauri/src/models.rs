use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub status: String,
    pub cover_relpath: Option<String>,
    pub cover_credit: Option<String>,
    pub cover_credit_url: Option<String>,
    pub cover_x: f64,
    pub cover_y: f64,
    pub sort: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverHit {
    pub id: String,
    pub thumb_relpath: String,
    pub credit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverPage {
    pub hits: Vec<CoverHit>,
    pub page: u32,
    pub page_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverCredit {
    pub credit: String,
    pub credit_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: String,
    pub name: String,
    pub iso: Option<String>,
    pub cover_relpath: Option<String>,
    pub cover_x: f64,
    pub cover_y: f64,
    pub own_cover: bool,
    pub sort: i32,
    pub created_at: String,
    pub place_count: i32,
    pub been_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub country_id: String,
    pub name: String,
    pub cover_relpath: Option<String>,
    pub cover_x: f64,
    pub cover_y: f64,
    pub notes: Option<String>,
    pub status: String,
    pub sort: i32,
    pub created_at: String,
    pub image_count: i32,
    pub when_text: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub id: String,
    pub name: String,
    pub when_text: Option<String>,
    pub year: Option<i32>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub cover_relpath: Option<String>,
    pub cover_credit: Option<String>,
    pub cover_credit_url: Option<String>,
    pub cover_x: f64,
    pub cover_y: f64,
    pub sort: i32,
    pub created_at: String,
    pub stop_count: i32,
    pub total: i64,
    pub cost_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripStop {
    pub place_id: String,
    pub country_id: String,
    pub place_name: String,
    pub country_name: String,
    pub status: String,
    pub sort: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripCost {
    pub category: String,
    pub amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripDetail {
    pub trip: Trip,
    pub stops: Vec<TripStop>,
    pub costs: Vec<TripCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceTask {
    pub id: String,
    pub place_id: String,
    pub body: String,
    pub done: bool,
    pub sort: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: String,
    pub place_id: String,
    pub original_relpath: String,
    pub display_relpath: String,
    pub thumb_relpath: String,
    pub sort: i32,
    pub sha256: Option<String>,
    pub caption: Option<String>,
    pub taken_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashItem {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub deleted_at: String,
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
pub struct SearchHit {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub country_id: Option<String>,
}
