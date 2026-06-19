use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ImageRecord {
    pub id: i64,
    pub source_path: String,
    pub filename: String,
    pub scan_id: Option<String>,
    pub file_size: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub title: Option<String>,
    pub status: String,
    pub organized_path: Option<String>,
    pub enhanced_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub phash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct ImageFilters {
    pub year: Option<i64>,
    pub month: Option<i64>,
    pub status: Option<String>,
    pub step: Option<String>,
    pub year_unknown: bool,
}

#[derive(Debug, Serialize)]
pub struct ImageListResponse {
    pub images: Vec<ImageRecord>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct FilterCountItem {
    pub value: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct FilterCounts {
    pub years: Vec<FilterCountItem>,
    pub months: Vec<FilterCountItem>,
    pub statuses: Vec<FilterCountItem>,
    pub steps: Vec<FilterCountItem>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSettings {
    pub source_dir: Option<String>,
    pub output_dir: Option<String>,
    pub thumbnail_size: u32,
}
