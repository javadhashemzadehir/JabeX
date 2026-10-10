//! Desktop shell concerns (system tray, window close behaviour).
//!
//! Kept outside domain/application so Tauri UI APIs do not leak into use-cases.

pub mod tray;

pub use tray::{install_tray, sync_tray_with_preferences, toggle_main_window, MAIN_WINDOW_LABEL};
