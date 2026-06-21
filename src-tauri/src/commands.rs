use tauri::ipc::{Channel, Response};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::models::{
    AppSettings, DuplicateGroup, FilterCounts, HistoryRecord, ImageFilters, ImageListResponse,
    ImageRecord, ModelEvent, ModelStatus, ProgressEvent, TaskOut,
};
use crate::state::AppState;
use crate::{db, hash, imaging, ml, organize, pipeline, scan, thumbnails};

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> CmdResult<AppSettings> {
    Ok(state.settings_dto())
}

#[tauri::command]
pub async fn pick_source_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Option<AppSettings>> {
    let Some(file) = app.dialog().file().blocking_pick_folder() else {
        log::info!("pick_source_folder: dialog cancelled");
        return Ok(None);
    };
    log::info!("pick_source_folder: selected {file:?}");
    let path = file.into_path().map_err(err)?;

    {
        let mut s = state.settings.lock().unwrap();
        s.source_dir = Some(path.clone());
    }
    db::set_setting(&state.db, "source_dir", &path.to_string_lossy())
        .await
        .map_err(err)?;

    Ok(Some(state.settings_dto()))
}

#[tauri::command]
pub async fn scan_source(state: State<'_, AppState>) -> CmdResult<usize> {
    let source = state.source_dir().ok_or("No source folder selected")?;
    scan::scan_source(&state.db, source).await.map_err(err)
}

#[tauri::command]
pub async fn list_images(
    state: State<'_, AppState>,
    filters: ImageFilters,
    page: i64,
    per_page: i64,
) -> CmdResult<ImageListResponse> {
    let (images, total) = db::list_images(&state.db, &filters, page, per_page)
        .await
        .map_err(err)?;
    Ok(ImageListResponse {
        images,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub async fn image_stats(
    state: State<'_, AppState>,
    filters: ImageFilters,
) -> CmdResult<FilterCounts> {
    db::image_stats(&state.db, &filters).await.map_err(err)
}

#[tauri::command]
pub async fn get_image(state: State<'_, AppState>, id: i64) -> CmdResult<Option<ImageRecord>> {
    db::get_image(&state.db, id).await.map_err(err)
}

#[tauri::command]
pub async fn get_thumbnail(state: State<'_, AppState>, id: i64) -> CmdResult<Response> {
    let img = db::get_image(&state.db, id)
        .await
        .map_err(err)?
        .ok_or("Image not found")?;
    let output_dir = state.output_dir();
    // Prefer the most-processed variant so the grid reflects pipeline results.
    let src = if let Some(p) = &img.enhanced_path {
        output_dir.join(p)
    } else if let Some(p) = &img.organized_path {
        output_dir.join(p)
    } else {
        let source = state.source_dir().ok_or("No source folder selected")?;
        source.join(&img.source_path)
    };
    let dest = state.thumbnails_dir().join(format!("{id}.jpg"));
    let size = state.thumbnail_size();

    let bytes = tauri::async_runtime::spawn_blocking(move || {
        thumbnails::ensure_thumbnail(&src, &dest, size)
    })
    .await
    .map_err(err)?
    .map_err(err)?;

    Ok(Response::new(bytes))
}

// ─── Pipeline ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_batch(
    state: State<'_, AppState>,
    image_ids: Vec<i64>,
    all: bool,
    steps: Vec<String>,
    on_event: Channel<ProgressEvent>,
) -> CmdResult<TaskOut> {
    let ids = if all {
        db::all_image_ids(&state.db).await.map_err(err)?
    } else {
        image_ids
    };
    if ids.is_empty() {
        return Err("No images selected".into());
    }
    if steps.is_empty() {
        return Err("No steps selected".into());
    }

    let steps_json = serde_json::to_string(&steps).map_err(err)?;
    let task_id = db::create_task(&state.db, &steps_json, ids.len() as i64)
        .await
        .map_err(err)?;
    for &id in &ids {
        db::add_task_item(&state.db, task_id, id).await.map_err(err)?;
    }

    if !state.try_start(task_id) {
        db::set_task_status(
            &state.db,
            task_id,
            "failed",
            Some("Another task is already running"),
        )
        .await
        .ok();
        return Err("A task is already running".into());
    }

    let result = pipeline::run(state.inner(), task_id, ids, steps, &on_event).await;
    state.finish();
    result.map_err(err)?;

    let task = db::get_task(&state.db, task_id)
        .await
        .map_err(err)?
        .ok_or("Task missing")?;
    Ok(task.into())
}

#[tauri::command]
pub async fn cancel_task(state: State<'_, AppState>, task_id: i64) -> CmdResult<bool> {
    Ok(state.request_cancel(task_id))
}

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>) -> CmdResult<Vec<TaskOut>> {
    let tasks = db::list_tasks(&state.db).await.map_err(err)?;
    Ok(tasks.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn get_task(state: State<'_, AppState>, id: i64) -> CmdResult<Option<TaskOut>> {
    Ok(db::get_task(&state.db, id).await.map_err(err)?.map(Into::into))
}

#[tauri::command]
pub async fn image_history(
    state: State<'_, AppState>,
    id: i64,
) -> CmdResult<Vec<HistoryRecord>> {
    db::list_history(&state.db, id).await.map_err(err)
}

// ─── Image mutations ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn rotate_image(
    state: State<'_, AppState>,
    id: i64,
    clockwise: bool,
) -> CmdResult<ImageRecord> {
    let img = db::get_image(&state.db, id)
        .await
        .map_err(err)?
        .ok_or("Image not found")?;
    let source_dir = state.source_dir().ok_or("No source folder selected")?;
    let output_dir = state.output_dir();

    // Rotation operates on the organized copy; create it if needed.
    let rel = match img.organized_path.clone().filter(|r| output_dir.join(r).exists()) {
        Some(r) => r,
        None => {
            let source_abs = source_dir.join(&img.source_path);
            let source_rel = img.source_path.clone();
            let od = output_dir.clone();
            let bn = pipeline::base_name(&img);
            let (year, month) = (img.year, img.month);
            let r = tauri::async_runtime::spawn_blocking(move || {
                organize::organize(&source_abs, &source_rel, &od, year, month, &bn)
            })
            .await
            .map_err(err)?
            .map_err(err)?;
            db::set_image_organized(&state.db, id, &r).await.map_err(err)?;
            r
        }
    };

    let abs = output_dir.join(&rel);
    let abs2 = abs.clone();
    tauri::async_runtime::spawn_blocking(move || imaging::rotate(&abs, clockwise))
        .await
        .map_err(err)?
        .map_err(err)?;

    if let Ok((w, h)) = image::image_dimensions(&abs2) {
        db::set_image_dimensions(&state.db, id, w as i64, h as i64)
            .await
            .map_err(err)?;
    }
    db::add_history(&state.db, id, if clockwise { "rotate_right" } else { "rotate_left" })
        .await
        .map_err(err)?;
    let _ = std::fs::remove_file(state.thumbnails_dir().join(format!("{id}.jpg")));

    db::get_image(&state.db, id)
        .await
        .map_err(err)?
        .ok_or_else(|| "Image not found".into())
}

#[tauri::command]
pub async fn update_image(
    state: State<'_, AppState>,
    id: i64,
    year: Option<i64>,
    month: Option<i64>,
    title: Option<String>,
) -> CmdResult<ImageRecord> {
    db::update_image_meta(&state.db, id, year, month, title, true, true, true)
        .await
        .map_err(err)?;
    db::get_image(&state.db, id)
        .await
        .map_err(err)?
        .ok_or_else(|| "Image not found".into())
}

#[tauri::command]
pub async fn bulk_update(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    year: Option<i64>,
    month: Option<i64>,
    title: Option<String>,
) -> CmdResult<u64> {
    let (sy, sm, st) = (year.is_some(), month.is_some(), title.is_some());
    db::bulk_update_meta(&state.db, &ids, year, month, title, sy, sm, st)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn bulk_delete(state: State<'_, AppState>, ids: Vec<i64>) -> CmdResult<u64> {
    let recs = db::fetch_images(&state.db, &ids).await.map_err(err)?;
    let output_dir = state.output_dir();
    let thumbs = state.thumbnails_dir();
    for r in &recs {
        if let Some(p) = &r.organized_path {
            let _ = std::fs::remove_file(output_dir.join(p));
        }
        if let Some(p) = &r.enhanced_path {
            let _ = std::fs::remove_file(output_dir.join(p));
        }
        let _ = std::fs::remove_file(thumbs.join(format!("{}.jpg", r.id)));
    }
    db::delete_images(&state.db, &ids).await.map_err(err)
}

#[tauri::command]
pub async fn get_variant(
    state: State<'_, AppState>,
    id: i64,
    variant: String,
) -> CmdResult<Response> {
    let img = db::get_image(&state.db, id)
        .await
        .map_err(err)?
        .ok_or("Image not found")?;
    let output_dir = state.output_dir();
    let path = match variant.as_str() {
        "enhanced" => img.enhanced_path.map(|p| output_dir.join(p)),
        "organized" => img.organized_path.map(|p| output_dir.join(p)),
        _ => state.source_dir().map(|s| s.join(&img.source_path)),
    }
    .ok_or("Variant not available")?;

    let bytes = tauri::async_runtime::spawn_blocking(move || std::fs::read(&path))
        .await
        .map_err(err)?
        .map_err(err)?;
    Ok(Response::new(bytes))
}

// ─── Duplicates ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn scan_duplicates(state: State<'_, AppState>) -> CmdResult<usize> {
    let source = state.source_dir().ok_or("No source folder selected")?;
    let pending = db::images_without_phash(&state.db).await.map_err(err)?;
    let mut done = 0;
    for (id, rel) in pending {
        let path = source.join(&rel);
        let h = tauri::async_runtime::spawn_blocking(move || hash::phash(&path))
            .await
            .map_err(err)?;
        if let Ok(ph) = h {
            db::set_phash(&state.db, id, &ph).await.map_err(err)?;
            done += 1;
        }
    }
    Ok(done)
}

fn group_duplicates(rows: Vec<(i64, String)>, threshold: u32) -> Vec<DuplicateGroup> {
    let mut used = vec![false; rows.len()];
    let mut groups = Vec::new();
    for i in 0..rows.len() {
        if used[i] {
            continue;
        }
        let mut ids = vec![rows[i].0];
        let mut max_dist = 0u32;
        for j in (i + 1)..rows.len() {
            if used[j] {
                continue;
            }
            if let Some(d) = hash::distance(&rows[i].1, &rows[j].1) {
                if d <= threshold {
                    ids.push(rows[j].0);
                    used[j] = true;
                    max_dist = max_dist.max(d);
                }
            }
        }
        if ids.len() > 1 {
            used[i] = true;
            groups.push(DuplicateGroup {
                image_ids: ids,
                distance: max_dist as i64,
            });
        }
    }
    groups
}

#[tauri::command]
pub async fn find_duplicates(
    state: State<'_, AppState>,
    threshold: i64,
) -> CmdResult<Vec<DuplicateGroup>> {
    let rows = db::images_with_phash(&state.db).await.map_err(err)?;
    let groups =
        tauri::async_runtime::spawn_blocking(move || group_duplicates(rows, threshold.max(0) as u32))
            .await
            .map_err(err)?;
    Ok(groups)
}

// ─── Settings ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pick_output_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<Option<AppSettings>> {
    let Some(file) = app.dialog().file().blocking_pick_folder() else {
        log::info!("pick_output_folder: dialog cancelled");
        return Ok(None);
    };
    log::info!("pick_output_folder: selected {file:?}");
    let path = file.into_path().map_err(err)?;
    {
        let mut s = state.settings.lock().unwrap();
        s.output_dir = Some(path.clone());
    }
    db::set_setting(&state.db, "output_dir", &path.to_string_lossy())
        .await
        .map_err(err)?;
    Ok(Some(state.settings_dto()))
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    thumbnail_size: u32,
) -> CmdResult<AppSettings> {
    {
        let mut s = state.settings.lock().unwrap();
        s.thumbnail_size = thumbnail_size.clamp(100, 1000);
    }
    let size = state.thumbnail_size();
    db::set_setting(&state.db, "thumbnail_size", &size.to_string())
        .await
        .map_err(err)?;
    Ok(state.settings_dto())
}

// ─── ML models ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> CmdResult<Vec<ModelStatus>> {
    Ok(ml::statuses(&state.models_dir()))
}

#[tauri::command]
pub async fn models_dir(state: State<'_, AppState>) -> CmdResult<String> {
    Ok(state.models_dir().to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn download_models(
    state: State<'_, AppState>,
    keys: Option<Vec<String>>,
    on_event: Channel<ModelEvent>,
) -> CmdResult<()> {
    let model_dir = state.models_dir();
    tauri::async_runtime::spawn_blocking(move || {
        ml::download_models(&model_dir, keys.as_deref(), &on_event);
    })
    .await
    .map_err(err)
}
