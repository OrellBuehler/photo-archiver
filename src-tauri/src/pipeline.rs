use std::path::Path;

use anyhow::{anyhow, Result};
use tauri::ipc::Channel;

use crate::models::{ImageRecord, ProgressEvent};
use crate::state::AppState;
use crate::{db, imaging, organize};

/// Canonical execution order; requested steps run in this order regardless of input order.
pub const STEP_ORDER: &[&str] = &[
    "organize",
    "orient",
    "crop",
    "auto_orient",
    "deskew",
    "restore_color",
    "remove_dust",
    "remove_lines",
    "enhance",
];

pub fn base_name(img: &ImageRecord) -> String {
    if let Some(s) = &img.scan_id {
        return s.clone();
    }
    Path::new(&img.filename)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| img.filename.clone())
}

/// Run a single classical step in place. Returns whether it was handled
/// (ONNX steps are wired in Phase 3 and return `false` here).
fn run_step_blocking(step: &str, path: &Path, model_dir: &Path) -> Result<bool> {
    match step {
        "orient" => imaging::orient(path).map(|_| true),
        "crop" => imaging::crop(path).map(|_| true),
        "deskew" => imaging::deskew(path).map(|_| true),
        "restore_color" => imaging::restore_color(path).map(|_| true),
        "remove_dust" => imaging::remove_dust(path).map(|_| true),
        "auto_orient" => crate::ml::orient(path, model_dir),
        "remove_lines" => crate::ml::remove_lines(path, model_dir),
        _ => Ok(false),
    }
}

async fn do_organize(
    state: &AppState,
    img: &ImageRecord,
    source_dir: &Path,
    output_dir: &Path,
) -> Result<String> {
    let source_abs = source_dir.join(&img.source_path);
    let source_rel = img.source_path.clone();
    let output_dir = output_dir.to_path_buf();
    let bn = base_name(img);
    let (year, month) = (img.year, img.month);
    let rel = tauri::async_runtime::spawn_blocking(move || {
        organize::organize(&source_abs, &source_rel, &output_dir, year, month, &bn)
    })
    .await??;
    db::set_image_organized(&state.db, img.id, &rel).await?;
    Ok(rel)
}

async fn process_image(
    state: &AppState,
    task_id: i64,
    img_id: i64,
    steps: &[String],
    source_dir: &Path,
    output_dir: &Path,
    channel: &Channel<ProgressEvent>,
) -> Result<()> {
    let img = db::get_image(&state.db, img_id)
        .await?
        .ok_or_else(|| anyhow!("image not found"))?;
    let mut organized_rel = img.organized_path.clone();
    let model_dir = state.data_dir.join(".models");

    for &step in STEP_ORDER {
        if !steps.iter().any(|s| s.as_str() == step) {
            continue;
        }
        let _ = channel.send(ProgressEvent::StepStarted {
            task_id,
            image_id: img_id,
            step: step.to_string(),
        });
        db::set_task_item(&state.db, task_id, img_id, "running", Some(step), None).await?;

        let handled = if step == "organize" {
            organized_rel = Some(do_organize(state, &img, source_dir, output_dir).await?);
            true
        } else {
            // All other steps operate on (or derive from) the organized copy.
            let rel = match &organized_rel {
                Some(r) if output_dir.join(r).exists() => r.clone(),
                _ => {
                    let r = do_organize(state, &img, source_dir, output_dir).await?;
                    organized_rel = Some(r.clone());
                    r
                }
            };
            if step == "enhance" {
                let organized_abs = output_dir.join(&rel);
                let enhanced_rel = rel.replacen("organized/", "enhanced/", 1);
                let enhanced_abs = output_dir.join(&enhanced_rel);
                let md = model_dir.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    crate::ml::enhance(&organized_abs, &enhanced_abs, &md)
                })
                .await??;
                db::set_image_enhanced(&state.db, img_id, &enhanced_rel).await?;
                true
            } else {
                let abs = output_dir.join(&rel);
                let step_owned = step.to_string();
                let md = model_dir.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    run_step_blocking(&step_owned, &abs, &md)
                })
                .await??
            }
        };

        if handled {
            db::add_history(&state.db, img_id, step).await?;
        }
        let _ = channel.send(ProgressEvent::StepCompleted {
            task_id,
            image_id: img_id,
            step: step.to_string(),
        });
    }

    if organized_rel.is_some() {
        let enhanced = db::get_image(&state.db, img_id)
            .await?
            .and_then(|i| i.enhanced_path)
            .is_some();
        let status = if enhanced { "enhanced" } else { "organized" };
        db::set_image_status(&state.db, img_id, status).await?;
    }
    // Drop the cached thumbnail so it regenerates from the new best variant.
    let _ = std::fs::remove_file(state.thumbnails_dir().join(format!("{img_id}.jpg")));
    Ok(())
}

/// Execute a task to completion, streaming progress over `channel`.
pub async fn run(
    state: &AppState,
    task_id: i64,
    image_ids: Vec<i64>,
    steps: Vec<String>,
    channel: &Channel<ProgressEvent>,
) -> Result<String> {
    let source_dir = state
        .source_dir()
        .ok_or_else(|| anyhow!("No source folder selected"))?;
    let output_dir = state.output_dir();
    let _ = std::fs::create_dir_all(&output_dir);

    let total = image_ids.len() as i64;
    db::set_task_status(&state.db, task_id, "running", None).await?;
    let _ = channel.send(ProgressEvent::TaskStarted { task_id, total });

    let mut completed = 0i64;
    let mut failed = 0i64;
    let mut cancelled = false;

    for img_id in image_ids {
        if state.is_cancelled() {
            cancelled = true;
            break;
        }
        let _ = channel.send(ProgressEvent::ImageStarted {
            task_id,
            image_id: img_id,
        });
        db::set_task_item(&state.db, task_id, img_id, "running", None, None).await?;

        match process_image(state, task_id, img_id, &steps, &source_dir, &output_dir, channel).await
        {
            Ok(()) => {
                completed += 1;
                db::set_task_item(&state.db, task_id, img_id, "completed", None, None).await?;
            }
            Err(e) => {
                failed += 1;
                let msg = e.to_string();
                db::set_task_item(&state.db, task_id, img_id, "failed", None, Some(&msg)).await?;
                let _ = channel.send(ProgressEvent::ImageFailed {
                    task_id,
                    image_id: img_id,
                    error: msg,
                });
            }
        }
        db::update_task_counts(&state.db, task_id, completed, failed).await?;
        let _ = channel.send(ProgressEvent::Progress {
            task_id,
            completed,
            failed,
            total,
        });
    }

    let status = if cancelled { "cancelled" } else { "completed" };
    db::set_task_status(&state.db, task_id, status, None).await?;
    let _ = channel.send(ProgressEvent::TaskCompleted {
        task_id,
        status: status.to_string(),
    });
    Ok(status.to_string())
}
