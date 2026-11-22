use std::path::Path;

use anyhow::{anyhow, Result};
use tauri::ipc::Channel;

use crate::models::{ImageRecord, ProgressEvent};
use crate::state::AppState;
use crate::{db, imaging, organize, snapshots};

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
    "restore_faces",
    "enhance",
];

pub fn base_name(img: &ImageRecord) -> String {
    let stem = if let Some(s) = &img.scan_id {
        s.clone()
    } else {
        Path::new(&img.filename)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| img.filename.clone())
    };
    // The id keeps the organized path unique (distinct sources can share a
    // stem/scan-id now that we ingest many formats) and stable across re-runs.
    format!("{stem}_{}", img.id)
}

/// Run a single in-place step. Returns whether it was handled.
fn run_step_blocking(step: &str, path: &Path, model_dir: &Path) -> Result<bool> {
    match step {
        "orient" => imaging::orient(path).map(|_| true),
        "crop" => imaging::crop(path).map(|_| true),
        "deskew" => imaging::deskew(path).map(|_| true),
        "restore_color" => imaging::restore_color(path).map(|_| true),
        "remove_dust" => imaging::remove_dust(path).map(|_| true),
        "auto_orient" => crate::ml::orient(path, model_dir),
        "remove_lines" => crate::ml::remove_lines(path, model_dir),
        "restore_faces" => crate::ml::restore_faces(path, model_dir),
        _ => Ok(false),
    }
}

pub async fn do_organize(
    state: &AppState,
    img: &ImageRecord,
    source_dir: &Path,
    output_dir: &Path,
) -> Result<String> {
    let source_abs = source_dir.join(&img.source_path);
    let source_rel = img.source_path.clone();
    let out_dir = output_dir.to_path_buf();
    let bn = base_name(img);
    let album = img.folder.clone();
    let (year, month) = (img.year, img.month);
    let rel = tauri::async_runtime::spawn_blocking(move || {
        organize::organize(
            &source_abs,
            &source_rel,
            &out_dir,
            album.as_deref(),
            year,
            month,
            &bn,
        )
    })
    .await??;
    db::set_image_organized(&state.db, img.id, &rel).await?;
    // Fresh organized copy → reset the undo stack to this as the base state.
    snapshots::reset(state, img.id, &output_dir.join(&rel)).await?;
    Ok(rel)
}

fn enhanced_rel(organized_rel: &str) -> String {
    organized_rel.replacen("organized/", "enhanced/", 1)
}

/// Move an already-filed photo (and its upscaled copy) to where its current
/// date and folder say it belongs, keeping every edit. Returns the organized
/// path if the photo has a copy on disk.
pub async fn refile(state: &AppState, img: &ImageRecord) -> Result<Option<String>> {
    let output_dir = state.output_dir();
    let Some(old) = img.organized_path.clone().filter(|r| output_dir.join(r).exists()) else {
        return Ok(None);
    };
    let new = organize::rel_path(img.folder.as_deref(), img.year, img.month, &base_name(img));
    if new == old {
        return Ok(Some(old));
    }
    organize::move_file(&output_dir, &old, &new)?;
    let enhanced = match img.enhanced_path.as_ref().filter(|e| output_dir.join(e).exists()) {
        Some(e) => {
            let to = enhanced_rel(&new);
            organize::move_file(&output_dir, e, &to)?;
            Some(to)
        }
        None => None,
    };
    db::set_image_paths(&state.db, img.id, &new, enhanced.as_deref()).await?;
    crate::exif::write_provenance(&output_dir.join(&new), &img.source_path, img.year, img.month);
    log::info!("refiled image {}: {old} -> {new}", img.id);
    Ok(Some(new))
}

/// The organized copy changed, so an upscaled copy made from it is stale.
pub async fn drop_enhanced(state: &AppState, img: &ImageRecord) -> Result<()> {
    if let Some(p) = &img.enhanced_path {
        let _ = std::fs::remove_file(state.output_dir().join(p));
        db::clear_image_enhanced(&state.db, img.id).await?;
    }
    Ok(())
}

/// Throw away everything made from a scan — filed copy, upscale, undo stack —
/// so it is an untouched new photo again. The scan itself is never touched.
pub async fn reset_image(state: &AppState, img: &ImageRecord) -> Result<()> {
    let output_dir = state.output_dir();
    for p in [&img.organized_path, &img.enhanced_path].into_iter().flatten() {
        let _ = std::fs::remove_file(output_dir.join(p));
    }
    db::take_all_snapshots(&state.db, img.id).await?;
    let _ = std::fs::remove_dir_all(output_dir.join(format!(".snapshots/{}", img.id)));
    let _ = std::fs::remove_file(state.thumbnails_dir().join(format!("{}.jpg", img.id)));
    db::clear_image_files(&state.db, img.id).await?;
    Ok(())
}

/// Bring one image up to date with the requested steps. Steps already baked
/// into its copy are skipped, so re-running a recipe only does what is missing;
/// `force` starts over from the scan instead.
async fn process_image(
    state: &AppState,
    task_id: i64,
    img_id: i64,
    steps: &[String],
    force: bool,
    source_dir: &Path,
    output_dir: &Path,
    channel: &Channel<ProgressEvent>,
) -> Result<()> {
    let fetch = || async {
        db::get_image(&state.db, img_id)
            .await?
            .ok_or_else(|| anyhow!("image not found"))
    };
    let mut img = fetch().await?;
    if force {
        reset_image(state, &img).await?;
        img = fetch().await?;
    }
    let mut organized_rel = img.organized_path.clone().filter(|r| output_dir.join(r).exists());
    let applied = match organized_rel {
        Some(_) => db::applied_steps(&state.db, img_id).await?,
        None => Vec::new(),
    };
    let model_dir = state.models_dir();
    let mut changed = false;
    let mut enhanced_now = false;

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

        // Every step works on the organized copy; make it if it is missing.
        let fresh = organized_rel.is_none();
        let rel = match organized_rel.clone() {
            Some(r) => r,
            None => {
                let r = do_organize(state, &img, source_dir, output_dir).await?;
                organized_rel = Some(r.clone());
                r
            }
        };

        let did = if step == "organize" {
            fresh
        } else if step == "enhance" {
            let dest_rel = enhanced_rel(&rel);
            let enhanced_abs = output_dir.join(&dest_rel);
            if !changed && img.enhanced_path.is_some() && enhanced_abs.exists() {
                false
            } else {
                let organized_abs = output_dir.join(&rel);
                let md = model_dir.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    crate::ml::enhance(&organized_abs, &enhanced_abs, &md)
                })
                .await??;
                db::set_image_enhanced(&state.db, img_id, &dest_rel).await?;
                enhanced_now = true;
                true
            }
        } else if applied.iter().any(|s| s == step) {
            false
        } else {
            let abs = output_dir.join(&rel);
            // Seed a base snapshot of the pre-step state if the stack is
            // empty (an image organized before the snapshot feature), so undo
            // can reach it.
            snapshots::ensure_base(state, img_id, &abs).await?;
            let abs_for_snap = abs.clone();
            let step_owned = step.to_string();
            let md = model_dir.clone();
            let did = tauri::async_runtime::spawn_blocking(move || {
                run_step_blocking(&step_owned, &abs, &md)
            })
            .await??;
            // Snapshot the new organized state so the step can be undone.
            if did {
                snapshots::record(state, img_id, step, &abs_for_snap).await?;
                changed = true;
            }
            did
        };

        if did {
            db::add_history(&state.db, img_id, step).await?;
        }
        let _ = channel.send(ProgressEvent::StepCompleted {
            task_id,
            image_id: img_id,
            step: step.to_string(),
        });
    }

    // Pick up date/folder edits (also ones made while this was running).
    let img = fetch().await?;
    if changed && !enhanced_now {
        drop_enhanced(state, &img).await?;
    }
    refile(state, &fetch().await?).await?;
    db::refresh_status(&state.db, img_id).await?;
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
    force: bool,
    channel: &Channel<ProgressEvent>,
) -> Result<String> {
    let source_dir = state
        .source_dir()
        .ok_or_else(|| anyhow!("No source folder selected"))?;
    let output_dir = state.output_dir();
    let _ = std::fs::create_dir_all(&output_dir);

    let total = image_ids.len() as i64;
    log::info!("task {task_id} starting: {total} image(s), steps {steps:?}, force {force}");
    state.set_busy(&image_ids);
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

        match process_image(state, task_id, img_id, &steps, force, &source_dir, &output_dir, channel)
            .await
        {
            Ok(()) => {
                completed += 1;
                db::set_task_item(&state.db, task_id, img_id, "completed", None, None).await?;
            }
            Err(e) => {
                failed += 1;
                let msg = e.to_string();
                log::error!("task {task_id} image {img_id} failed: {msg}");
                db::set_task_item(&state.db, task_id, img_id, "failed", None, Some(&msg)).await?;
                db::set_image_failed(&state.db, img_id, &msg).await?;
                let _ = std::fs::remove_file(state.thumbnails_dir().join(format!("{img_id}.jpg")));
                let _ = channel.send(ProgressEvent::ImageFailed {
                    task_id,
                    image_id: img_id,
                    error: msg,
                });
            }
        }
        state.release(img_id);
        db::update_task_counts(&state.db, task_id, completed, failed).await?;
        let _ = channel.send(ProgressEvent::Progress {
            task_id,
            completed,
            failed,
            total,
        });
    }

    let status = if cancelled { "cancelled" } else { "completed" };
    log::info!("task {task_id} {status}: {completed} done, {failed} failed");
    db::set_task_status(&state.db, task_id, status, None).await?;
    let _ = channel.send(ProgressEvent::TaskCompleted {
        task_id,
        status: status.to_string(),
    });
    Ok(status.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;

    use image::{Rgb, RgbImage};

    use crate::state::Settings;

    async fn setup(root: &Path) -> AppState {
        let (source, data) = (root.join("scans"), root.join("data"));
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(data.join("thumbnails")).unwrap();
        // A photo on a white scanner bed, so cropping has something to do.
        let mut img = RgbImage::from_pixel(240, 180, Rgb([255, 255, 255]));
        for (x, y, p) in img.enumerate_pixels_mut() {
            if (30..210).contains(&x) && (30..150).contains(&y) {
                *p = Rgb([(x % 200) as u8, (y % 200) as u8, 90]);
            }
        }
        crate::imaging::save_rgb(&img, &source.join("SCAN_0001.jpg")).unwrap();
        let db = db::init_pool(&data).await.unwrap();
        crate::scan::scan_source(&db, source.clone()).await.unwrap();
        AppState {
            db,
            settings: Mutex::new(Settings {
                source_dir: Some(source),
                output_dir: Some(root.join("archive")),
                thumbnail_size: 200,
            }),
            data_dir: data,
            running: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            current_task: Mutex::new(None),
            busy: Mutex::new(HashSet::new()),
        }
    }

    async fn restore(state: &AppState, steps: &[&str], force: bool) {
        let steps: Vec<String> = steps.iter().map(|s| s.to_string()).collect();
        let task = db::create_task(&state.db, "[]", 1).await.unwrap();
        db::add_task_items(&state.db, task, &[1]).await.unwrap();
        let channel = Channel::new(|_| Ok(()));
        run(state, task, vec![1], steps, force, &channel).await.unwrap();
        state.finish();
    }

    #[test]
    fn rerun_skips_done_steps_and_edits_refile() {
        let root: PathBuf = std::env::temp_dir().join(format!("pa_pipeline_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        tauri::async_runtime::block_on(async {
            let state = setup(&root).await;
            let out = state.output_dir();
            let get = || async { db::get_image(&state.db, 1).await.unwrap().unwrap() };
            assert_eq!(get().await.status, "new");

            restore(&state, &["organize"], false).await;
            let img = get().await;
            assert_eq!(img.status, "organized");
            assert_eq!(img.organized_path.as_deref(), Some("organized/unsorted/SCAN_0001_1.jpg"));

            restore(&state, &["organize", "crop"], false).await;
            let img = get().await;
            assert_eq!(img.status, "restored");
            assert_eq!(db::applied_steps(&state.db, 1).await.unwrap(), ["crop"]);
            let cropped = image::image_dimensions(out.join(img.organized_path.unwrap())).unwrap();
            assert!(cropped.0 < 240 && cropped.1 < 180);

            // Same recipe again: nothing is re-copied or re-cropped.
            restore(&state, &["organize", "crop"], false).await;
            assert_eq!(db::applied_steps(&state.db, 1).await.unwrap(), ["crop"]);
            assert_eq!(db::list_snapshots(&state.db, 1).await.unwrap().len(), 1);

            // Dating the photo moves the cropped copy; no orphan stays behind.
            db::update_image_meta(&state.db, 1, Some(1978), Some(6), None, true, true, false)
                .await
                .unwrap();
            refile(&state, &get().await).await.unwrap();
            let img = get().await;
            let rel = img.organized_path.clone().unwrap();
            assert_eq!(rel, "organized/1978/1978-06_SCAN_0001_1.jpg");
            assert_eq!(image::image_dimensions(out.join(&rel)).unwrap(), cropped);
            assert!(!out.join("organized/unsorted").exists());

            // A failure is remembered on the photo and cleared by the next good run.
            db::set_image_failed(&state.db, 1, "boom").await.unwrap();
            restore(&state, &["organize", "crop"], false).await;
            let img = get().await;
            assert_eq!((img.status.as_str(), img.error), ("restored", None));

            // Starting over goes back to the full scan.
            restore(&state, &["organize"], true).await;
            let img = get().await;
            assert_eq!(img.status, "organized");
            assert!(db::applied_steps(&state.db, 1).await.unwrap().is_empty());
            assert_eq!(image::image_dimensions(out.join(img.organized_path.clone().unwrap())).unwrap(), (240, 180));

            reset_image(&state, &img).await.unwrap();
            let img = get().await;
            assert_eq!((img.status.as_str(), img.organized_path), ("new", None));
            assert!(!out.join(&rel).exists());
            state.db.close().await;
        });
        std::fs::remove_dir_all(&root).unwrap();
    }
}
