//! Per-image undo/redo stack. Each snapshot is a full JPEG of the organized
//! file at a point in time, stored under `<output>/.snapshots/<image_id>/<seq>.jpg`.
//! `images.history_pos` points at the currently-active seq.
//!
//! Snapshotting is best-effort: a failed file copy simply skips that snapshot
//! rather than failing the surrounding pipeline step.

use std::path::Path;

use anyhow::Result;

use crate::db;
use crate::state::AppState;

fn snap_rel(image_id: i64, seq: i64) -> String {
    format!(".snapshots/{image_id}/{seq}.jpg")
}

fn copy_into(output_dir: &Path, rel: &str, src: &Path) -> bool {
    let abs = output_dir.join(rel);
    if let Some(parent) = abs.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::copy(src, &abs).is_ok()
}

/// Reset the stack to a single base snapshot (`seq 0`) of the current organized
/// file. Called whenever the organized copy is (re)created from source.
pub async fn reset(state: &AppState, image_id: i64, organized_abs: &Path) -> Result<()> {
    let output_dir = state.output_dir();
    for p in db::take_all_snapshots(&state.db, image_id).await.unwrap_or_default() {
        let _ = std::fs::remove_file(output_dir.join(p));
    }
    let rel = snap_rel(image_id, 0);
    if copy_into(&output_dir, &rel, organized_abs) {
        db::add_snapshot(&state.db, image_id, 0, "base", &rel).await?;
    }
    db::set_history_pos(&state.db, image_id, 0).await?;
    Ok(())
}

/// Ensure a base snapshot exists for an image whose organized copy predates the
/// snapshot feature, seeding `seq 0` from `organized_abs` if the stack is empty.
pub async fn ensure_base(state: &AppState, image_id: i64, organized_abs: &Path) -> Result<()> {
    if db::max_snapshot_seq(&state.db, image_id).await? < 0 {
        reset(state, image_id, organized_abs).await?;
    }
    Ok(())
}

/// Append a snapshot of the current organized file as the new tip, discarding
/// any redo branch above the current position.
pub async fn record(
    state: &AppState,
    image_id: i64,
    label: &str,
    organized_abs: &Path,
) -> Result<()> {
    let output_dir = state.output_dir();
    let pos = db::get_history_pos(&state.db, image_id).await?;
    for p in db::take_snapshots_after(&state.db, image_id, pos).await.unwrap_or_default() {
        let _ = std::fs::remove_file(output_dir.join(p));
    }
    let next = pos + 1;
    let rel = snap_rel(image_id, next);
    if copy_into(&output_dir, &rel, organized_abs) {
        db::add_snapshot(&state.db, image_id, next, label, &rel).await?;
        db::set_history_pos(&state.db, image_id, next).await?;
    }
    Ok(())
}

/// Restore the organized file to the snapshot at `seq`, returning whether it
/// succeeded. The caller is responsible for updating `history_pos`.
pub async fn restore_to(
    state: &AppState,
    image_id: i64,
    seq: i64,
    organized_abs: &Path,
) -> Result<bool> {
    let output_dir = state.output_dir();
    let Some(rel) = db::snapshot_path_at(&state.db, image_id, seq).await? else {
        return Ok(false);
    };
    let src = output_dir.join(rel);
    if !src.exists() {
        return Ok(false);
    }
    std::fs::copy(&src, organized_abs)?;
    Ok(true)
}
