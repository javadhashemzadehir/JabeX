use crate::domain::app_preferences::AppPreferences;
use crate::domain::errors::{JabeXError, JabeXResult};
use crate::infrastructure::autostart;
use crate::infrastructure::registry::ProfileRegistry;

pub struct AppPreferencesService {
    registry: ProfileRegistry,
}

impl AppPreferencesService {
    pub fn new(registry: ProfileRegistry) -> Self {
        Self { registry }
    }

    pub fn get(&self) -> JabeXResult<AppPreferences> {
        match self.registry.get_meta(AppPreferences::META_KEY)? {
            Some(raw) => serde_json::from_str(&raw)
                .map_err(|e| JabeXError::Internal(format!("invalid app preferences JSON: {e}"))),
            None => Ok(AppPreferences::default()),
        }
    }

    pub fn set(&self, prefs: AppPreferences) -> JabeXResult<AppPreferences> {
        // Keep OS registration in sync with the stored flag (best-effort on non-Windows).
        autostart::set_launch_on_startup(prefs.launch_on_startup)?;

        let encoded =
            serde_json::to_string(&prefs).map_err(|e| JabeXError::Internal(e.to_string()))?;
        self.registry.set_meta(AppPreferences::META_KEY, &encoded)?;
        tracing::info!(
            tray = prefs.enable_minimize_to_tray,
            close_to_tray = prefs.close_button_hides_to_tray,
            start_minimized = prefs.start_minimized,
            launch_on_startup = prefs.launch_on_startup,
            "app preferences updated"
        );
        Ok(prefs)
    }
}
