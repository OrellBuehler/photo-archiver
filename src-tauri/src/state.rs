use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    pub current_task: Mutex<Option<i64>>,
}

impl AppState {
    /// Snapshot the source dir without holding the lock across awaits.
    pub fn source_dir(&self) -> Option<PathBuf> {
        self.settings.lock().unwrap().source_dir.clone()
    }

    /// Configured output dir, or a default under the app data dir.
    pub fn output_dir(&self) -> PathBuf {
        self.settings
            .lock()
            .unwrap()
            .output_dir
            .clone()
            .unwrap_or_else(|| self.data_dir.join("output"))
    }

    pub fn thumbnail_size(&self) -> u32 {
        self.settings.lock().unwrap().thumbnail_size
    }

    pub fn settings_dto(&self) -> AppSettings {
        let s = self.settings.lock().unwrap();
        // Derive the output dir from this same guard — calling self.output_dir()
        // here would re-lock the non-reentrant mutex and deadlock.
        let output_dir = s
            .output_dir
            .clone()
            .unwrap_or_else(|| self.data_dir.join("output"));
        AppSettings {
            source_dir: s.source_dir.as_ref().map(|p| p.to_string_lossy().into_owned()),
            output_dir: Some(output_dir.to_string_lossy().into_owned()),
            thumbnail_size: s.thumbnail_size,
        }
    }

    pub fn thumbnails_dir(&self) -> PathBuf {
        self.data_dir.join("thumbnails")
    }

    /// Try to claim the single pipeline slot. Returns false if one is running.
    pub fn try_start(&self, task_id: i64) -> bool {
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return false;
        }
        self.cancel.store(false, Ordering::SeqCst);
        *self.current_task.lock().unwrap() = Some(task_id);
        true
    }

    pub fn finish(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.cancel.store(false, Ordering::SeqCst);
        *self.current_task.lock().unwrap() = None;
    }

    pub fn request_cancel(&self, task_id: i64) -> bool {
        if *self.current_task.lock().unwrap() == Some(task_id) {
            self.cancel.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}
