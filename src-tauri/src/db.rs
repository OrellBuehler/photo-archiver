use std::path::Path;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::models::{
    FilterCountItem, FilterCounts, HistoryRecord, ImageFilters, ImageRecord, TaskRecord,
};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS images (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_path TEXT NOT NULL UNIQUE,
  filename TEXT NOT NULL,
  scan_id TEXT,
  file_size INTEGER,
  width INTEGER,
  height INTEGER,
  year INTEGER,
  month INTEGER,
  title TEXT,
  status TEXT NOT NULL DEFAULT 'source',
  organized_path TEXT,
  enhanced_path TEXT,
  thumbnail_path TEXT,
  phash TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  status TEXT NOT NULL DEFAULT 'pending',
  steps TEXT NOT NULL,
  total_images INTEGER NOT NULL DEFAULT 0,
  completed_images INTEGER NOT NULL DEFAULT 0,
  failed_images INTEGER NOT NULL DEFAULT 0,
  error_message TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  started_at TEXT,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS task_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
  status TEXT NOT NULL DEFAULT 'pending',
  current_step TEXT,
  error_message TEXT,
  started_at TEXT,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS image_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
  step TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;

pub async fn init_pool(data_dir: &Path) -> Result<SqlitePool> {
    let db_path = data_dir.join("photo-archiver.db");
    let opts = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    sqlx::query(SCHEMA).execute(&pool).await?;
    Ok(pool)
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

fn apply_filters(qb: &mut QueryBuilder<'_, Sqlite>, f: &ImageFilters) {
    qb.push(" WHERE 1 = 1");
    if f.year_unknown {
        qb.push(" AND year IS NULL");
    } else if let Some(y) = f.year {
        qb.push(" AND year = ").push_bind(y);
    }
    if let Some(m) = f.month {
        qb.push(" AND month = ").push_bind(m);
    }
    if let Some(s) = &f.status {
        qb.push(" AND status = ").push_bind(s.clone());
    }
    if let Some(step) = &f.step {
        qb.push(" AND EXISTS (SELECT 1 FROM image_history h WHERE h.image_id = images.id AND h.step = ")
            .push_bind(step.clone())
            .push(")");
    }
}

pub async fn list_images(
    pool: &SqlitePool,
    f: &ImageFilters,
    page: i64,
    per_page: i64,
) -> Result<(Vec<ImageRecord>, i64)> {
    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM images");
    apply_filters(&mut count_qb, f);
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let offset = (page - 1).max(0) * per_page;
    let mut qb = QueryBuilder::new("SELECT * FROM images");
    apply_filters(&mut qb, f);
    qb.push(" ORDER BY year IS NULL, year DESC, month IS NULL, month DESC, filename ASC");
    qb.push(" LIMIT ").push_bind(per_page);
    qb.push(" OFFSET ").push_bind(offset);
    let images = qb.build_query_as::<ImageRecord>().fetch_all(pool).await?;

    Ok((images, total))
}

pub async fn get_image(pool: &SqlitePool, id: i64) -> Result<Option<ImageRecord>> {
    let rec = sqlx::query_as::<_, ImageRecord>("SELECT * FROM images WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(rec)
}

pub async fn image_stats(pool: &SqlitePool, f: &ImageFilters) -> Result<FilterCounts> {
    // Years: every distinct year across the library, plus an "unknown" bucket.
    let mut years: Vec<FilterCountItem> =
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT year, COUNT(*) FROM images WHERE year IS NOT NULL GROUP BY year ORDER BY year DESC",
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(y, c)| FilterCountItem { value: y.to_string(), count: c })
        .collect();
    let unknown: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM images WHERE year IS NULL")
        .fetch_one(pool)
        .await?;
    if unknown > 0 {
        years.push(FilterCountItem { value: "unknown".into(), count: unknown });
    }

    // Months: only relevant within a selected year.
    let months: Vec<FilterCountItem> = if let Some(y) = f.year {
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT month, COUNT(*) FROM images WHERE year = ? AND month IS NOT NULL GROUP BY month ORDER BY month",
        )
        .bind(y)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|(m, c)| FilterCountItem { value: m.to_string(), count: c })
        .collect()
    } else {
        Vec::new()
    };

    let statuses: Vec<FilterCountItem> =
        sqlx::query_as::<_, (String, i64)>("SELECT status, COUNT(*) FROM images GROUP BY status")
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|(s, c)| FilterCountItem { value: s, count: c })
            .collect();

    let steps: Vec<FilterCountItem> = sqlx::query_as::<_, (String, i64)>(
        "SELECT step, COUNT(DISTINCT image_id) FROM image_history GROUP BY step ORDER BY step",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(s, c)| FilterCountItem { value: s, count: c })
    .collect();

    let mut total_qb = QueryBuilder::new("SELECT COUNT(*) FROM images");
    apply_filters(&mut total_qb, f);
    let total: i64 = total_qb.build_query_scalar().fetch_one(pool).await?;

    Ok(FilterCounts { years, months, statuses, steps, total })
}

// ─── Tasks ────────────────────────────────────────────────────────────────────

pub async fn create_task(pool: &SqlitePool, steps_json: &str, total: i64) -> Result<i64> {
    let id = sqlx::query("INSERT INTO tasks (status, steps, total_images) VALUES ('pending', ?, ?)")
        .bind(steps_json)
        .bind(total)
        .execute(pool)
        .await?
        .last_insert_rowid();
    Ok(id)
}

pub async fn add_task_item(pool: &SqlitePool, task_id: i64, image_id: i64) -> Result<()> {
    sqlx::query("INSERT INTO task_items (task_id, image_id) VALUES (?, ?)")
        .bind(task_id)
        .bind(image_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_tasks(pool: &SqlitePool) -> Result<Vec<TaskRecord>> {
    Ok(
        sqlx::query_as::<_, TaskRecord>("SELECT * FROM tasks ORDER BY created_at DESC, id DESC")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_task(pool: &SqlitePool, id: i64) -> Result<Option<TaskRecord>> {
    Ok(sqlx::query_as::<_, TaskRecord>("SELECT * FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn set_task_status(
    pool: &SqlitePool,
    id: i64,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    match status {
        "running" => {
            sqlx::query("UPDATE tasks SET status = ?, started_at = datetime('now') WHERE id = ?")
                .bind(status)
                .bind(id)
                .execute(pool)
                .await?;
        }
        "completed" | "failed" | "cancelled" => {
            sqlx::query(
                "UPDATE tasks SET status = ?, error_message = ?, completed_at = datetime('now') WHERE id = ?",
            )
            .bind(status)
            .bind(error)
            .bind(id)
            .execute(pool)
            .await?;
        }
        _ => {
            sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
                .bind(status)
                .bind(id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn update_task_counts(
    pool: &SqlitePool,
    id: i64,
    completed: i64,
    failed: i64,
) -> Result<()> {
    sqlx::query("UPDATE tasks SET completed_images = ?, failed_images = ? WHERE id = ?")
        .bind(completed)
        .bind(failed)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_task_item(
    pool: &SqlitePool,
    task_id: i64,
    image_id: i64,
    status: &str,
    current_step: Option<&str>,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE task_items SET status = ?, current_step = ?, error_message = ? WHERE task_id = ? AND image_id = ?",
    )
    .bind(status)
    .bind(current_step)
    .bind(error)
    .bind(task_id)
    .bind(image_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── History & image mutations ────────────────────────────────────────────────

pub async fn add_history(pool: &SqlitePool, image_id: i64, step: &str) -> Result<()> {
    sqlx::query("INSERT INTO image_history (image_id, step) VALUES (?, ?)")
        .bind(image_id)
        .bind(step)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_history(pool: &SqlitePool, image_id: i64) -> Result<Vec<HistoryRecord>> {
    Ok(sqlx::query_as::<_, HistoryRecord>(
        "SELECT * FROM image_history WHERE image_id = ? ORDER BY created_at, id",
    )
    .bind(image_id)
    .fetch_all(pool)
    .await?)
}

pub async fn set_image_status(pool: &SqlitePool, id: i64, status: &str) -> Result<()> {
    sqlx::query("UPDATE images SET status = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_image_organized(pool: &SqlitePool, id: i64, path: &str) -> Result<()> {
    sqlx::query("UPDATE images SET organized_path = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_image_enhanced(pool: &SqlitePool, id: i64, path: &str) -> Result<()> {
    sqlx::query("UPDATE images SET enhanced_path = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_image_dimensions(pool: &SqlitePool, id: i64, w: i64, h: i64) -> Result<()> {
    sqlx::query("UPDATE images SET width = ?, height = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(w)
        .bind(h)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_image_meta(
    pool: &SqlitePool,
    id: i64,
    year: Option<i64>,
    month: Option<i64>,
    title: Option<String>,
    set_year: bool,
    set_month: bool,
    set_title: bool,
) -> Result<()> {
    if !set_year && !set_month && !set_title {
        return Ok(());
    }
    let mut qb = QueryBuilder::new("UPDATE images SET updated_at = datetime('now')");
    if set_year {
        qb.push(", year = ").push_bind(year);
    }
    if set_month {
        qb.push(", month = ").push_bind(month);
    }
    if set_title {
        qb.push(", title = ").push_bind(title);
    }
    qb.push(" WHERE id = ").push_bind(id);
    qb.build().execute(pool).await?;
    Ok(())
}

pub async fn bulk_update_meta(
    pool: &SqlitePool,
    ids: &[i64],
    year: Option<i64>,
    month: Option<i64>,
    title: Option<String>,
    set_year: bool,
    set_month: bool,
    set_title: bool,
) -> Result<u64> {
    if ids.is_empty() || (!set_year && !set_month && !set_title) {
        return Ok(0);
    }
    let mut qb = QueryBuilder::new("UPDATE images SET updated_at = datetime('now')");
    if set_year {
        qb.push(", year = ").push_bind(year);
    }
    if set_month {
        qb.push(", month = ").push_bind(month);
    }
    if set_title {
        qb.push(", title = ").push_bind(title);
    }
    qb.push(" WHERE id IN (");
    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(*id);
    }
    qb.push(")");
    let res = qb.build().execute(pool).await?;
    Ok(res.rows_affected())
}

pub async fn fetch_images(pool: &SqlitePool, ids: &[i64]) -> Result<Vec<ImageRecord>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut qb = QueryBuilder::new("SELECT * FROM images WHERE id IN (");
    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(*id);
    }
    qb.push(")");
    Ok(qb.build_query_as::<ImageRecord>().fetch_all(pool).await?)
}

pub async fn delete_images(pool: &SqlitePool, ids: &[i64]) -> Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut qb = QueryBuilder::new("DELETE FROM images WHERE id IN (");
    let mut sep = qb.separated(", ");
    for id in ids {
        sep.push_bind(*id);
    }
    qb.push(")");
    let res = qb.build().execute(pool).await?;
    Ok(res.rows_affected())
}

pub async fn all_image_ids(pool: &SqlitePool) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar("SELECT id FROM images ORDER BY id")
        .fetch_all(pool)
        .await?)
}
