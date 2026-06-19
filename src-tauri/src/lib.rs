mod commands;
mod db;
mod exif;
mod hash;
mod imaging;
mod ml;
mod models;
mod organize;
mod pipeline;
mod scan;
mod state;
mod thumbnails;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use tauri::Manager;

use state::{AppState, Settings};

#[tauri::command]
fn ping() -> String {
    "pong".into()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            std::fs::create_dir_all(data_dir.join("thumbnails"))?;

            let pool =
                tauri::async_runtime::block_on(db::init_pool(&data_dir)).map_err(|e| e.to_string())?;

            let source_dir = tauri::async_runtime::block_on(db::get_setting(&pool, "source_dir"))
                .map_err(|e| e.to_string())?
                .map(PathBuf::from);
            let output_dir = tauri::async_runtime::block_on(db::get_setting(&pool, "output_dir"))
                .map_err(|e| e.to_string())?
                .map(PathBuf::from);
            let thumbnail_size =
                tauri::async_runtime::block_on(db::get_setting(&pool, "thumbnail_size"))
                    .map_err(|e| e.to_string())?
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(400);

            app.manage(AppState {
                db: pool,
                settings: Mutex::new(Settings {
                    source_dir,
                    output_dir,
                    thumbnail_size,
                }),
                data_dir,
                running: AtomicBool::new(false),
                cancel: AtomicBool::new(false),
                current_task: Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            commands::get_settings,
            commands::pick_source_folder,
            commands::scan_source,
            commands::list_images,
            commands::image_stats,
            commands::get_image,
            commands::get_thumbnail,
            commands::start_batch,
            commands::cancel_task,
            commands::list_tasks,
            commands::get_task,
            commands::image_history,
            commands::rotate_image,
            commands::update_image,
            commands::bulk_update,
            commands::bulk_delete,
            commands::get_variant,
            commands::scan_duplicates,
            commands::find_duplicates,
            commands::pick_output_folder,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
