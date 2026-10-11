use serde::{Deserialize, Serialize};

/// Device-level keyboard shortcut chords (e.g. `mod+k`, `space`, `1`).
///
/// Stored with app preferences so they stay stable across local profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardShortcuts {
    #[serde(default = "default_open_search")]
    pub open_search: String,
    #[serde(default = "default_review_flip_or_good")]
    pub review_flip_or_good: String,
    #[serde(default = "default_review_flip_or_good_alt")]
    pub review_flip_or_good_alt: String,
    #[serde(default = "default_review_grade_again")]
    pub review_grade_again: String,
    #[serde(default = "default_review_grade_hard")]
    pub review_grade_hard: String,
    #[serde(default = "default_review_grade_good")]
    pub review_grade_good: String,
    #[serde(default = "default_review_grade_easy")]
    pub review_grade_easy: String,
    #[serde(default = "default_review_play_audio")]
    pub review_play_audio: String,
}

fn default_open_search() -> String {
    "mod+k".into()
}
fn default_review_flip_or_good() -> String {
    "space".into()
}
fn default_review_flip_or_good_alt() -> String {
    "enter".into()
}
fn default_review_grade_again() -> String {
    "1".into()
}
fn default_review_grade_hard() -> String {
    "2".into()
}
fn default_review_grade_good() -> String {
    "3".into()
}
fn default_review_grade_easy() -> String {
    "4".into()
}
fn default_review_play_audio() -> String {
    "p".into()
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            open_search: default_open_search(),
            review_flip_or_good: default_review_flip_or_good(),
            review_flip_or_good_alt: default_review_flip_or_good_alt(),
            review_grade_again: default_review_grade_again(),
            review_grade_hard: default_review_grade_hard(),
            review_grade_good: default_review_grade_good(),
            review_grade_easy: default_review_grade_easy(),
            review_play_audio: default_review_play_audio(),
        }
    }
}

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
    /// Customizable keyboard shortcuts (defaults applied when missing).
    #[serde(default)]
    pub keyboard_shortcuts: KeyboardShortcuts,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            // Sensible Windows desktop defaults: stay available from the tray.
            enable_minimize_to_tray: true,
            close_button_hides_to_tray: true,
            start_minimized: false,
            launch_on_startup: false,
            keyboard_shortcuts: KeyboardShortcuts::default(),
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

    #[test]
    fn deserializes_legacy_prefs_without_shortcuts() {
        let json = r#"{
            "enableMinimizeToTray": true,
            "closeButtonHidesToTray": true,
            "startMinimized": false,
            "launchOnStartup": false
        }"#;
        let prefs: AppPreferences = serde_json::from_str(json).expect("deserialize");
        assert_eq!(prefs.keyboard_shortcuts.open_search, "mod+k");
        assert_eq!(prefs.keyboard_shortcuts.review_grade_good, "3");
    }
}
