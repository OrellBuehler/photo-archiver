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
mod snapshots;
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
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri_plugin_log::{Target, TargetKind};
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    // Persist to a rotating file in the OS log dir so users can
                    // send it for debugging, while still mirroring to stdout.
                    .max_file_size(5_000_000)
                    .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
                    .targets([
                        Target::new(TargetKind::Stdout),
                        Target::new(TargetKind::LogDir {
                            file_name: Some("photo-archiver".into()),
                        }),
                    ])
                    .build(),
            )?;

            // Route panics into the log so crashes are captured in the file too.
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                log::error!("panic: {info}");
                default_hook(info);
            }));

            if let Ok(log_dir) = app.path().app_log_dir() {
                log::info!(
                    "photo-archiver v{} starting; logs at {}",
                    env!("CARGO_PKG_VERSION"),
                    log_dir.join("photo-archiver.log").display()
                );
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
            commands::set_source_folder,
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
            commands::bulk_rotate,
            commands::update_image,
            commands::bulk_update,
            commands::bulk_delete,
            commands::set_folder,
            commands::undo_image,
            commands::redo_image,
            commands::snapshot_state,
            commands::get_variant,
            commands::scan_duplicates,
            commands::find_duplicates,
            commands::pick_output_folder,
            commands::update_settings,
            commands::list_models,
            commands::models_dir,
            commands::log_dir,
            commands::open_log_dir,
            commands::download_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
