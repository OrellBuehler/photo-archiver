use std::path::PathBuf;
use std::sync::Mutex;

use sqlx::SqlitePool;

use crate::models::AppSettings;

pub struct Settings {
    pub source_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub thumbnail_size: u32,
}

pub struct AppState {
    pub db: SqlitePool,
    pub settings: Mutex<Settings>,
    pub data_dir: PathBuf,
}

impl AppState {
    /// Snapshot the source dir without holding the lock across awaits.
    pub fn source_dir(&self) -> Option<PathBuf> {
        self.settings.lock().unwrap().source_dir.clone()
    }

    pub fn thumbnail_size(&self) -> u32 {
        self.settings.lock().unwrap().thumbnail_size
    }

    pub fn settings_dto(&self) -> AppSettings {
        let s = self.settings.lock().unwrap();
        AppSettings {
            source_dir: s.source_dir.as_ref().map(|p| p.to_string_lossy().into_owned()),
            output_dir: s.output_dir.as_ref().map(|p| p.to_string_lossy().into_owned()),
            thumbnail_size: s.thumbnail_size,
        }
    }

    pub fn thumbnails_dir(&self) -> PathBuf {
        self.data_dir.join("thumbnails")
    }
}
