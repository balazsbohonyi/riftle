// Phase 6: Launch commands — launch(id), launch_elevated(id) via ShellExecuteW

use tauri::Manager;

#[tauri::command]
pub fn launch(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // DB lookup in scoped block so MutexGuard drops before rebuild_index.
    // Bind state to a local variable first to satisfy the borrow checker (temporary lifetime).
    let path = {
        let db_state = app.state::<crate::db::DbState>();
        let conn = db_state.0.lock().unwrap();
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

    // Hide window BEFORE ShellExecuteW (LAUN-04 — hide-first for perceived performance)
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
        // Success — increment launch count then rebuild search index
        {
            let db_state = app.state::<crate::db::DbState>();
            let conn = db_state.0.lock().unwrap();
            let _ = crate::db::increment_launch_count(&conn, &id);
        }
        crate::search::rebuild_index(&app);
    } else {
        eprintln!("[launch] ShellExecuteW failed: {}", result as isize);
    }

    Ok(())
}

#[tauri::command]
pub fn launch_elevated(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // DB lookup in scoped block so MutexGuard drops before rebuild_index.
    // Bind state to a local variable first to satisfy the borrow checker (temporary lifetime).
    let path = {
        let db_state = app.state::<crate::db::DbState>();
        let conn = db_state.0.lock().unwrap();
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
            // UAC cancelled — window stays open (it was never hidden)
            return Ok(());
        }
        eprintln!("[launch_elevated] ShellExecuteW error: {}", err);
        return Ok(());
    }

    // Success — hide window, then increment launch count and rebuild index
    if let Some(win) = app.get_webview_window("launcher") {
        let _ = win.hide();
    }
    {
        let db_state = app.state::<crate::db::DbState>();
        let conn = db_state.0.lock().unwrap();
        let _ = crate::db::increment_launch_count(&conn, &id);
    }
    crate::search::rebuild_index(&app);

    Ok(())
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
        // Empty string — just the null terminator
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
}
