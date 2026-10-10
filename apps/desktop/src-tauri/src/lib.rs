pub mod application;
pub mod commands;
pub mod desktop;
pub mod domain;
pub mod infrastructure;
pub mod shared;
pub mod state;

use tracing_subscriber::EnvFilter;

use crate::desktop::tray::{install_tray, on_close_requested};
use crate::infrastructure::database::open_database;
use crate::state::AppState;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tracing::info!("starting JabeX backend");

    let (app_paths, db, active_profile_id) = match open_database() {
        Ok(ready) => ready,
        Err(err) => {
            tracing::error!(error = %err, "failed to initialize database");
            panic!("failed to initialize database: {err}");
        }
    };
    let state = match AppState::new(app_paths, db, active_profile_id) {
        Ok(state) => state,
        Err(err) => {
            tracing::error!(error = %err, "failed to initialize app state");
            panic!("failed to initialize app state: {err}");
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
            // Tray is created once here; preference changes rebuild via sync_tray_with_preferences.
            if let Err(err) = install_tray(app) {
                tracing::error!(error = %err, "failed to install system tray");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                on_close_requested(window, api);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::get_active_profile,
            commands::create_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::switch_profile,
            commands::create_deck,
            commands::list_decks,
            commands::get_deck,
            commands::rename_deck,
            commands::delete_deck,
            commands::create_card,
            commands::get_card,
            commands::list_cards,
            commands::update_card,
            commands::delete_card,
            commands::list_due_cards,
            commands::submit_review,
            commands::get_setting,
            commands::set_setting,
            commands::get_app_preferences,
            commands::set_app_preferences,
            commands::export_backup,
            commands::create_backup,
            commands::validate_backup,
            commands::import_backup,
            commands::store_media,
        ])
        .run(tauri::generate_context!())
        .expect("error while running JabeX");
}
