// Phase 6: Launch commands - launch(id), launch_elevated(id) via ShellExecuteW

use std::path::{Path, PathBuf};
use std::sync::{MutexGuard, PoisonError};
use std::time::Duration;
use tauri::Manager;

const GENERIC_ICON_FILENAME: &str = "generic.png";

fn lock_db<'a>(
    db_state: &'a crate::db::DbState,
    context: &str,
) -> MutexGuard<'a, rusqlite::Connection> {
    db_state.0.lock().unwrap_or_else(|err: PoisonError<_>| {
        eprintln!("[{context}] recovering from poisoned DB mutex");
        err.into_inner()
    })
}

#[tauri::command]
pub fn launch(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // DB lookup in scoped block so MutexGuard drops before rebuild_index.
    // Bind state to a local variable first to satisfy the borrow checker (temporary lifetime).
    let path = {
        let db_state = app.state::<crate::db::DbState>();
        let conn = lock_db(&db_state, "launch");
        conn.query_row(
            "SELECT path FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        )
        .ok()
    };
    let Some(path) = path else {
        eprintln!("[launch] app not found: {}", id);
        return Ok(());
    };

    // Hide window BEFORE ShellExecuteW (LAUN-04 - hide-first for perceived performance)
    if let Some(win) = app.get_webview_window("launcher") {
        let _ = win.hide();
    }

    // Launch via ShellExecuteW with NULL verb (default open)
    let file = to_wide_null(&path);
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            std::ptr::null(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };

    if result as isize > 32 {
        // Success - increment launch count then rebuild search index
        {
            let db_state = app.state::<crate::db::DbState>();
            let conn = lock_db(&db_state, "launch");
            let _ = crate::db::increment_launch_count(&conn, &id);
        }
        crate::search::rebuild_index(&app);
    } else {
        eprintln!("[launch] ShellExecuteW failed: {}", result as isize);
    }

    Ok(())
}

fn icon_path_under_data_dir(data_dir: &Path, filename: &str) -> Result<PathBuf, String> {
    if !crate::search::validate_icon_filename(filename) {
        return Err("invalid icon filename".to_string());
    }

    Ok(data_dir.join("icons").join(filename))
}

pub fn read_icon_bytes_from_data_dir(data_dir: &Path, filename: &str) -> Result<Vec<u8>, String> {
    let requested_path = icon_path_under_data_dir(data_dir, filename)?;
    match std::fs::read(&requested_path) {
        Ok(bytes) => Ok(bytes),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && filename != GENERIC_ICON_FILENAME => {
            let fallback_path = icon_path_under_data_dir(data_dir, GENERIC_ICON_FILENAME)?;
            std::fs::read(&fallback_path)
                .map_err(|fallback_err| format!("failed to read fallback icon: {fallback_err}"))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Err(format!("icon not found: {filename}"))
        }
        Err(err) => Err(format!("failed to read icon: {err}")),
    }
}

#[tauri::command]
pub fn get_icon_bytes(
    icon_path: String,
    data_dir: tauri::State<PathBuf>,
) -> Result<Vec<u8>, String> {
    read_icon_bytes_from_data_dir(data_dir.inner().as_path(), &icon_path)
}

#[tauri::command]
pub fn launch_elevated(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // DB lookup in scoped block so MutexGuard drops before rebuild_index.
    // Bind state to a local variable first to satisfy the borrow checker (temporary lifetime).
    let path = {
        let db_state = app.state::<crate::db::DbState>();
        let conn = lock_db(&db_state, "launch_elevated");
        conn.query_row(
            "SELECT path FROM apps WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        )
        .ok()
    };
    let Some(path) = path else {
        eprintln!("[launch_elevated] app not found: {}", id);
        return Ok(());
    };

    // For launch_elevated, do NOT hide window before the call.
    // UAC cancellation must leave the launcher open (LAUN-02).
    // Window is hidden ONLY after a successful launch (result > 32).
    let file = to_wide_null(&path);
    let runas = to_wide_null("runas");
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            runas.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
        )
    };

    if result as isize <= 32 {
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        const ERROR_CANCELLED: u32 = 1223;
        if err == ERROR_CANCELLED {
            // UAC cancelled - window stays open (it was never hidden)
            return Ok(());
        }
        eprintln!("[launch_elevated] ShellExecuteW error: {}", err);
        return Ok(());
    }

    // Success - hide window, then increment launch count and rebuild index
    if let Some(win) = app.get_webview_window("launcher") {
        let _ = win.hide();
    }
    {
        let db_state = app.state::<crate::db::DbState>();
        let conn = lock_db(&db_state, "launch_elevated");
        let _ = crate::db::increment_launch_count(&conn, &id);
    }
    crate::search::rebuild_index(&app);

    Ok(())
}

/// Phase 7: Quit the launcher process cleanly.
/// Explicit cleanup helps WebView teardown finish before process exit.
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    // Ask webviews to close first, then exit shortly after to reduce
    // WebView2/Chromium teardown race noise on Windows shutdown.
    for label in ["settings", "launcher"] {
        if let Some(win) = app.get_webview_window(label) {
            let _ = win.hide();
            let _ = win.close();
        }
    }

    let app_for_exit = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(120));
        app_for_exit.cleanup_before_exit();
        app_for_exit.exit(0);
    });
}

/// Converts a Rust &str to a null-terminated wide string (Vec<u16>) for Win32 API calls.
fn to_wide_null(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_icon(dir: &Path, filename: &str, bytes: &[u8]) {
        let icons_dir = dir.join("icons");
        std::fs::create_dir_all(&icons_dir).unwrap();
        std::fs::write(icons_dir.join(filename), bytes).unwrap();
    }

    #[test]
    fn test_to_wide_null_hello() {
        let result = to_wide_null("hello");
        // "hello" = h, e, l, l, o = 5 chars + 1 null terminator = length 6
        assert_eq!(result.len(), 6);
        assert_eq!(result[0], 'h' as u16);
        assert_eq!(result[1], 'e' as u16);
        assert_eq!(result[2], 'l' as u16);
        assert_eq!(result[3], 'l' as u16);
        assert_eq!(result[4], 'o' as u16);
        assert_eq!(result[5], 0); // null terminator
    }

    #[test]
    fn test_to_wide_null_empty() {
        let result = to_wide_null("");
        // Empty string - just the null terminator
        assert_eq!(result, vec![0u16]);
    }

    #[test]
    fn test_to_wide_null_path() {
        let path = "C:\\path\\app.exe";
        let result = to_wide_null(path);
        // Length should be char count + 1 null terminator
        let expected_len = path.chars().count() + 1;
        assert_eq!(result.len(), expected_len);
        // Last element must be null terminator
        assert_eq!(*result.last().unwrap(), 0u16);
    }

    #[test]
    fn test_icon_path_under_data_dir_resolves_icons_subdir() {
        let temp = tempdir().unwrap();
        let path = icon_path_under_data_dir(temp.path(), "0123456789abcdef.png").unwrap();
        assert_eq!(path, temp.path().join("icons").join("0123456789abcdef.png"));
    }

    #[test]
    fn test_read_icon_bytes_rejects_invalid_filename() {
        let temp = tempdir().unwrap();
        let err = read_icon_bytes_from_data_dir(temp.path(), "..\\evil.exe").unwrap_err();
        assert_eq!(err, "invalid icon filename");
    }

    #[test]
    fn test_read_icon_bytes_returns_requested_icon() {
        let temp = tempdir().unwrap();
        write_icon(temp.path(), GENERIC_ICON_FILENAME, b"generic");
        write_icon(temp.path(), "0123456789abcdef.png", b"custom");

        let bytes = read_icon_bytes_from_data_dir(temp.path(), "0123456789abcdef.png").unwrap();
        assert_eq!(bytes, b"custom");
    }

    #[test]
    fn test_read_icon_bytes_missing_uses_generic_fallback() {
        let temp = tempdir().unwrap();
        write_icon(temp.path(), GENERIC_ICON_FILENAME, b"generic");

        let bytes = read_icon_bytes_from_data_dir(temp.path(), "0123456789abcdef.png").unwrap();
        assert_eq!(bytes, b"generic");
    }

    #[test]
    fn test_read_icon_bytes_missing_generic_fails_cleanly() {
        let temp = tempdir().unwrap();
        let err = read_icon_bytes_from_data_dir(temp.path(), GENERIC_ICON_FILENAME).unwrap_err();
        assert_eq!(err, "icon not found: generic.png");
    }
}
