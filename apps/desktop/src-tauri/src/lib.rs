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
        .plugin(tauri_plugin_dialog::init())
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
            commands::is_initial_profile_setup_pending,
            commands::complete_initial_profile_setup,
            commands::create_deck,
            commands::list_decks,
            commands::get_deck,
            commands::rename_deck,
            commands::update_deck_settings,
            commands::delete_deck,
            commands::list_leitner_presets,
            commands::get_profile_algorithm,
            commands::set_profile_algorithm,
            commands::apply_profile_algorithm_to_all_decks,
            commands::get_effective_algorithm,
            commands::create_card,
            commands::get_card,
            commands::list_cards,
            commands::update_card,
            commands::delete_card,
            commands::get_language_assist_settings,
            commands::set_language_assist_settings,
            commands::regenerate_language_assist,
            commands::read_pronunciation_audio,
            commands::set_card_language_assist,
            commands::list_due_cards,
            commands::submit_review,
            commands::get_study_stats,
            commands::get_study_consistency_stats,
            commands::list_study_activity,
            commands::get_app_preferences,
            commands::set_app_preferences,
            commands::export_backup,
            commands::validate_backup,
            commands::import_backup,
            commands::create_attachment,
            commands::list_attachments,
            commands::delete_attachment,
            commands::restore_attachment,
            commands::read_attachment_bytes,
            commands::list_media_library,
            commands::get_media_library_stats,
            commands::purge_attachment,
            commands::purge_deleted_attachments,
            commands::remove_orphan_media_dirs,
            commands::verify_media_library,
            commands::create_ink_document,
            commands::get_ink_document,
            commands::list_ink_documents,
            commands::save_ink_document,
            commands::delete_ink_document,
            commands::list_deleted_items,
            commands::restore_item,
            commands::permanently_delete_item,
            commands::search_global,
            commands::rebuild_search_index,
            commands::list_recent_searches,
            commands::clear_recent_searches,
            commands::delete_recent_search,
            commands::list_popular_searches,
            commands::record_search_selection,
            commands::list_search_history,
            commands::clear_search_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running JabeX");
}
