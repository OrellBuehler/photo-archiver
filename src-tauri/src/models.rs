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

#[derive(Debug, Clone, FromRow)]
pub struct TaskRecord {
    pub id: i64,
    pub status: String,
    pub steps: String,
    pub total_images: i64,
    pub completed_images: i64,
    pub failed_images: i64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskOut {
    pub id: i64,
    pub status: String,
    pub steps: Vec<String>,
    pub total_images: i64,
    pub completed_images: i64,
    pub failed_images: i64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl From<TaskRecord> for TaskOut {
    fn from(t: TaskRecord) -> Self {
        let steps = serde_json::from_str(&t.steps).unwrap_or_default();
        TaskOut {
            id: t.id,
            status: t.status,
            steps,
            total_images: t.total_images,
            completed_images: t.completed_images,
            failed_images: t.failed_images,
            error_message: t.error_message,
            created_at: t.created_at,
            started_at: t.started_at,
            completed_at: t.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub image_ids: Vec<i64>,
    pub distance: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct HistoryRecord {
    pub id: i64,
    pub image_id: i64,
    pub step: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub key: String,
    pub file: String,
    pub label: String,
    pub approx_mb: u32,
    pub downloaded: bool,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelEvent {
    Started { key: String, label: String },
    Progress { key: String, downloaded: u64, total: Option<u64> },
    Finished { key: String },
    Failed { key: String, error: String },
    AllDone,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    TaskStarted { task_id: i64, total: i64 },
    ImageStarted { task_id: i64, image_id: i64 },
    StepStarted { task_id: i64, image_id: i64, step: String },
    StepCompleted { task_id: i64, image_id: i64, step: String },
    Progress { task_id: i64, completed: i64, failed: i64, total: i64 },
    ImageFailed { task_id: i64, image_id: i64, error: String },
    TaskCompleted { task_id: i64, status: String },
}
