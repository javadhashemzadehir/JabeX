use serde::{Deserialize, Serialize};

/// Device-level application preferences (not per-profile study settings).
///
/// Stored in `app.sqlite` `app_meta` so tray/window behaviour stays stable
/// when the user switches local profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferences {
    /// When true, a single system-tray icon is shown and the app may run hidden.
    pub enable_minimize_to_tray: bool,
    /// When true (and tray is enabled), the window close (X) button hides instead of quitting.
    pub close_button_hides_to_tray: bool,
    /// When true, the main window starts hidden (tray must be enabled).
    pub start_minimized: bool,
    /// When true on Windows, register JabeX under the current-user Run key.
    pub launch_on_startup: bool,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            // Sensible Windows desktop defaults: stay available from the tray.
            enable_minimize_to_tray: true,
            close_button_hides_to_tray: true,
            start_minimized: false,
            launch_on_startup: false,
        }
    }
}

impl AppPreferences {
    pub const META_KEY: &'static str = "app_preferences";

    /// Tray + close-to-tray only make sense together.
    pub fn should_hide_on_close(&self) -> bool {
        self.enable_minimize_to_tray && self.close_button_hides_to_tray
    }

    pub fn should_start_hidden(&self) -> bool {
        self.enable_minimize_to_tray && self.start_minimized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_on_close_requires_tray() {
        let mut prefs = AppPreferences::default();
        assert!(prefs.should_hide_on_close());
        prefs.enable_minimize_to_tray = false;
        assert!(!prefs.should_hide_on_close());
    }

    #[test]
    fn start_hidden_requires_tray() {
        let mut prefs = AppPreferences {
            start_minimized: true,
            ..AppPreferences::default()
        };
        assert!(prefs.should_start_hidden());
        prefs.enable_minimize_to_tray = false;
        assert!(!prefs.should_start_hidden());
    }
}
