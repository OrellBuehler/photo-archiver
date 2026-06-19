use tauri::ipc::Response;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::db;
use crate::models::{AppSettings, FilterCounts, ImageFilters, ImageListResponse, ImageRecord};
use crate::scan;
use crate::state::AppState;
use crate::thumbnails;

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
    let picked = app.dialog().file().blocking_pick_folder();
    let Some(path) = picked.and_then(|f| f.as_path().map(|p| p.to_path_buf())) else {
        return Ok(None);
    };

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
    let source = state.source_dir().ok_or("No source folder selected")?;
    let src = source.join(&img.source_path);
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
