use std::path::Path;

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::models::{FilterCountItem, FilterCounts, ImageFilters, ImageRecord};

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

    let mut total_qb = QueryBuilder::new("SELECT COUNT(*) FROM images");
    apply_filters(&mut total_qb, f);
    let total: i64 = total_qb.build_query_scalar().fetch_one(pool).await?;

    Ok(FilterCounts { years, months, statuses, steps: Vec::new(), total })
}
