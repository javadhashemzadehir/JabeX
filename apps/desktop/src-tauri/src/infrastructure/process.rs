//! Helpers for spawning child processes from the GUI host.
//!
//! On Windows, console subsystem tools (PowerShell, `where`, CLI binaries) flash a
//! visible console unless `CREATE_NO_WINDOW` is set.

use std::ffi::OsStr;
use std::process::Command;

/// Same as [`Command::new`], but never shows a console window on Windows.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    hide_console(&mut cmd);
    cmd
}

fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Win32 CREATE_NO_WINDOW — run without allocating a console.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}
