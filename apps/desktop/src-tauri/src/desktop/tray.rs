//! System tray + main-window visibility for JabeX.
//!
//! Design notes:
//! - One tray icon id (`jabex-tray`) so rebuilds never leave duplicates.
//! - Left-click toggles window visibility; right-click opens the menu.
//! - Close (X) may hide instead of quit when preferences say so — Exit is explicit.
//! - Menu actions that need UI emit `tray://action` for the frontend to handle.

use std::sync::Mutex;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime, WebviewWindow, Wry,
};

use crate::domain::app_preferences::AppPreferences;
use crate::state::AppState;

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const TRAY_ID: &str = "jabex-tray";
pub const TRAY_ACTION_EVENT: &str = "tray://action";

/// Live preferences + optional tray handle (single instance).
pub struct TrayRuntime {
    pub preferences: Mutex<AppPreferences>,
    tray: Mutex<Option<TrayIcon<Wry>>>,
}

impl TrayRuntime {
    pub fn new(preferences: AppPreferences) -> Self {
        Self {
            preferences: Mutex::new(preferences),
            tray: Mutex::new(None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayMenuAction {
    Open,
    SwitchProfile,
    CreateBackup,
    RestoreBackup,
    Settings,
    Exit,
}

impl TrayMenuAction {
    fn id(self) -> &'static str {
        match self {
            Self::Open => "tray.open",
            Self::SwitchProfile => "tray.switch_profile",
            Self::CreateBackup => "tray.create_backup",
            Self::RestoreBackup => "tray.restore_backup",
            Self::Settings => "tray.settings",
            Self::Exit => "tray.exit",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "tray.open" => Some(Self::Open),
            "tray.switch_profile" => Some(Self::SwitchProfile),
            "tray.create_backup" => Some(Self::CreateBackup),
            "tray.restore_backup" => Some(Self::RestoreBackup),
            "tray.settings" => Some(Self::Settings),
            "tray.exit" => Some(Self::Exit),
            _ => None,
        }
    }
}

pub fn install_tray(app: &App) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let prefs = state
        .app_preferences
        .get()
        .unwrap_or_else(|_| AppPreferences::default());

    app.manage(TrayRuntime::new(prefs.clone()));

    if prefs.enable_minimize_to_tray {
        build_tray(app.handle())?;
    }

    if prefs.should_start_hidden() {
        if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            let _ = window.hide();
        }
    }

    Ok(())
}

/// Recreate or remove the tray after preferences change (idempotent).
pub fn sync_tray_with_preferences(app: &AppHandle, prefs: &AppPreferences) -> tauri::Result<()> {
    if let Some(runtime) = app.try_state::<TrayRuntime>() {
        if let Ok(mut guard) = runtime.preferences.lock() {
            *guard = prefs.clone();
        }
    }

    // Drop the previous icon first so the notification area never shows duplicates.
    remove_tray(app);

    if prefs.enable_minimize_to_tray {
        build_tray(app)?;
    } else {
        // Avoid trapping the user with a hidden window and no tray restore path.
        let _ = show_main_window(app);
    }
    Ok(())
}

fn remove_tray(app: &AppHandle) {
    if let Some(runtime) = app.try_state::<TrayRuntime>() {
        if let Ok(mut guard) = runtime.tray.lock() {
            *guard = None;
        }
    }
    let _ = app.remove_tray_by_id(TRAY_ID);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon().cloned() else {
        tracing::warn!("default window icon missing; system tray not created");
        return Ok(());
    };

    let open = MenuItem::with_id(
        app,
        TrayMenuAction::Open.id(),
        "Open JabeX",
        true,
        None::<&str>,
    )?;
    let switch_profile = MenuItem::with_id(
        app,
        TrayMenuAction::SwitchProfile.id(),
        "Switch Profile",
        true,
        None::<&str>,
    )?;
    let create_backup = MenuItem::with_id(
        app,
        TrayMenuAction::CreateBackup.id(),
        "Create Backup",
        true,
        None::<&str>,
    )?;
    let restore_backup = MenuItem::with_id(
        app,
        TrayMenuAction::RestoreBackup.id(),
        "Restore Backup",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        TrayMenuAction::Settings.id(),
        "Settings",
        true,
        None::<&str>,
    )?;
    let sep = PredefinedMenuItem::separator(app)?;
    let exit = MenuItem::with_id(
        app,
        TrayMenuAction::Exit.id(),
        "Exit JabeX",
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &open,
            &switch_profile,
            &create_backup,
            &restore_backup,
            &settings,
            &sep,
            &exit,
        ],
    )?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("JabeX")
        .menu(&menu)
        // Left-click toggles the window; menu is for right-click (Windows convention).
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Some(action) = TrayMenuAction::from_id(event.id.as_ref()) {
                handle_menu_action(app, action);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = toggle_main_window(app);
            }
        })
        .build(app)?;

    if let Some(runtime) = app.try_state::<TrayRuntime>() {
        if let Ok(mut guard) = runtime.tray.lock() {
            *guard = Some(tray);
        }
    }

    tracing::info!(id = TRAY_ID, "system tray installed");
    Ok(())
}

fn handle_menu_action(app: &AppHandle, action: TrayMenuAction) {
    match action {
        TrayMenuAction::Open => {
            let _ = show_main_window(app);
        }
        TrayMenuAction::Exit => {
            // Explicit quit — only path that fully terminates while close-to-tray is on.
            tracing::info!("exit requested from system tray");
            app.exit(0);
        }
        TrayMenuAction::CreateBackup => {
            let _ = show_main_window(app);
            match app.state::<AppState>().export_backup(None) {
                Ok(path) => {
                    tracing::info!(path = %path.display(), "backup created from tray");
                    let _ = app.emit(
                        TRAY_ACTION_EVENT,
                        serde_json::json!({
                            "action": "backupCreated",
                            "path": path.to_string_lossy(),
                        }),
                    );
                }
                Err(err) => {
                    tracing::error!(error = %err, "tray backup failed");
                    let _ = app.emit(
                        TRAY_ACTION_EVENT,
                        serde_json::json!({
                            "action": "backupFailed",
                            "message": err.to_string(),
                        }),
                    );
                }
            }
        }
        TrayMenuAction::SwitchProfile
        | TrayMenuAction::RestoreBackup
        | TrayMenuAction::Settings => {
            let _ = show_main_window(app);
            // Typed tray → UI contract: frontend opens existing flows (no profile logic here).
            let name = match action {
                TrayMenuAction::SwitchProfile => "switch-profile",
                TrayMenuAction::RestoreBackup => "restoreBackup",
                TrayMenuAction::Settings => "settings",
                _ => unreachable!(),
            };
            let _ = app.emit(TRAY_ACTION_EVENT, serde_json::json!({ "action": name }));
        }
    }
}

pub fn toggle_main_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };
    if window.is_visible().unwrap_or(false) {
        // Visible → hide (stay running in tray).
        window.hide()?;
    } else {
        show_window(&window)?;
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        show_window(&window)?;
    }
    Ok(())
}

fn show_window<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    // Unminimize first so restore works after a normal taskbar minimize.
    let _ = window.unminimize();
    window.show()?;
    window.set_focus()?;
    Ok(())
}

/// CloseRequested handler: hide when preferences say so; otherwise allow quit.
pub fn on_close_requested(window: &tauri::Window, api: &tauri::CloseRequestApi) {
    let hide = window
        .app_handle()
        .try_state::<TrayRuntime>()
        .and_then(|runtime| {
            runtime
                .preferences
                .lock()
                .ok()
                .map(|p| p.should_hide_on_close())
        })
        .unwrap_or(false);

    if hide {
        // Prevent destroy so the process + tray stay alive.
        api.prevent_close();
        let _ = window.hide();
        tracing::debug!("close button hid window to tray");
    }
}
