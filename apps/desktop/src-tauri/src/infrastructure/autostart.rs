//! Optional launch-on-startup for Windows (HKCU Run key).
//! Other platforms store the preference but do not register a launcher yet.

use crate::domain::errors::{JabeXError, JabeXResult};

const RUN_VALUE_NAME: &str = "JabeX";

/// Enable or disable launching JabeX when the user signs in.
pub fn set_launch_on_startup(enabled: bool) -> JabeXResult<()> {
    #[cfg(windows)]
    {
        windows_set_launch_on_startup(enabled)
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        // Preference is persisted; OS integration is Windows-first for now.
        Ok(())
    }
}

/// Whether the OS currently has JabeX registered for autostart.
#[allow(dead_code)]
pub fn is_launch_on_startup_enabled() -> JabeXResult<bool> {
    #[cfg(windows)]
    {
        windows_is_launch_on_startup_enabled()
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

#[cfg(windows)]
fn windows_set_launch_on_startup(enabled: bool) -> JabeXResult<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_READ | KEY_SET_VALUE,
        )
        .map_err(|e| JabeXError::Internal(format!("autostart registry open failed: {e}")))?;

    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| JabeXError::Internal(format!("failed to resolve executable path: {e}")))?;
        // Quote the path so spaces in Program Files do not break the Run key.
        let value = format!("\"{}\"", exe.display());
        key.set_value(RUN_VALUE_NAME, &value)
            .map_err(|e| JabeXError::Internal(format!("autostart registry write failed: {e}")))?;
        tracing::info!(path = %exe.display(), "registered launch on startup");
    } else {
        match key.delete_value(RUN_VALUE_NAME) {
            Ok(()) => tracing::info!("unregistered launch on startup"),
            // Already absent — treat as success for idempotent disable.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(JabeXError::Internal(format!(
                    "autostart registry delete failed: {e}"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn windows_is_launch_on_startup_enabled() -> JabeXResult<bool> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_READ)
    {
        Ok(key) => key,
        Err(_) => return Ok(false),
    };
    let value: Result<String, _> = key.get_value(RUN_VALUE_NAME);
    Ok(value.is_ok())
}
