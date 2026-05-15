// Phase 6: Launch commands - launch(id), launch_elevated(id) via ShellExecuteW

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{MutexGuard, PoisonError};
use std::time::Duration;
use tauri::Manager;

const GENERIC_ICON_FILENAME: &str = "generic.png";
const SE_ERR_NOASSOC_CODE: isize = 31;
const SHELL_SUCCESS_MIN_CODE: isize = 32;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShortcutLaunchResult {
    pub success: bool,
    pub warning: Option<crate::warnings::BackendWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShortcutTargetKind {
    Directory,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShortcutLaunchRequest {
    kind: ShortcutTargetKind,
    path: String,
    parameters: Option<String>,
    is_parameter_capable_executable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShortcutShellFailureAction {
    OpenWith,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShortcutCommandOutcome {
    result: ShortcutLaunchResult,
    should_hide_launcher: bool,
}

fn lock_db<'a>(
    db_state: &'a crate::db::DbState,
    context: &str,
) -> MutexGuard<'a, rusqlite::Connection> {
    db_state.0.lock().unwrap_or_else(|err: PoisonError<_>| {
        eprintln!("[{context}] recovering from poisoned DB mutex");
        err.into_inner()
    })
}

fn shortcut_launch_warning(title: &str, message: String) -> crate::warnings::BackendWarning {
    crate::warnings::BackendWarning {
        kind: "shortcut-launch-failed".to_string(),
        title: title.to_string(),
        message,
        backup_path: None,
    }
}

fn shortcut_launch_failure(title: &str, message: String) -> ShortcutLaunchResult {
    ShortcutLaunchResult {
        success: false,
        warning: Some(shortcut_launch_warning(title, message)),
    }
}

fn shortcut_launch_success() -> ShortcutLaunchResult {
    ShortcutLaunchResult {
        success: true,
        warning: None,
    }
}

fn shortcut_launch_request(
    kind: ShortcutTargetKind,
    path: &str,
    raw_parameters: &str,
) -> ShortcutLaunchRequest {
    let trimmed_parameters = raw_parameters.trim();
    let is_parameter_capable_executable = kind == ShortcutTargetKind::File
        && crate::shortcuts::is_parameterized_executable_target(path);
    let parameters = if is_parameter_capable_executable && !trimmed_parameters.is_empty() {
        Some(raw_parameters.to_string())
    } else {
        None
    };

    ShortcutLaunchRequest {
        kind,
        path: path.to_string(),
        parameters,
        is_parameter_capable_executable,
    }
}

fn resolve_shortcut_from_settings(
    settings: &crate::store::Settings,
    id: &str,
) -> Option<ShortcutLaunchRequest> {
    for shortcut in &settings.directory_shortcuts {
        if crate::shortcuts::shortcut_id("dir", &shortcut.path) == id {
            return Some(shortcut_launch_request(
                ShortcutTargetKind::Directory,
                &shortcut.path,
                "",
            ));
        }
    }

    for shortcut in &settings.file_shortcuts {
        if crate::shortcuts::shortcut_id("file", &shortcut.path) == id {
            return Some(shortcut_launch_request(
                ShortcutTargetKind::File,
                &shortcut.path,
                &shortcut.parameters,
            ));
        }
    }

    None
}

fn evaluate_shortcut_target_policy(request: &ShortcutLaunchRequest) -> ShortcutLaunchResult {
    if !Path::new(&request.path).exists() {
        return shortcut_launch_failure(
            "Shortcut target is missing",
            format!("Shortcut target does not exist: {}", request.path),
        );
    }

    shortcut_launch_success()
}

fn shortcut_shell_failed(shell_result: isize) -> bool {
    shell_result <= SHELL_SUCCESS_MIN_CODE
}

fn shortcut_shell_failure_action(
    request: &ShortcutLaunchRequest,
    shell_result: isize,
) -> ShortcutShellFailureAction {
    if shell_result == SE_ERR_NOASSOC_CODE
        && request.kind == ShortcutTargetKind::File
        && !request.is_parameter_capable_executable
    {
        ShortcutShellFailureAction::OpenWith
    } else {
        ShortcutShellFailureAction::Warn
    }
}

fn shortcut_shell_policy_result(
    request: &ShortcutLaunchRequest,
    shell_result: isize,
) -> ShortcutLaunchResult {
    if !shortcut_shell_failed(shell_result) {
        return shortcut_launch_success();
    }

    shortcut_launch_failure(
        "Shortcut launch failed",
        format!(
            "Windows could not open shortcut target {} (ShellExecuteW code {}).",
            request.path, shell_result
        ),
    )
}

fn shortcut_command_outcome(result: ShortcutLaunchResult) -> ShortcutCommandOutcome {
    ShortcutCommandOutcome {
        result,
        should_hide_launcher: false,
    }
}

fn shortcut_command_outcome_from_target_policy(
    request: &ShortcutLaunchRequest,
) -> ShortcutCommandOutcome {
    shortcut_command_outcome(evaluate_shortcut_target_policy(request))
}

fn shortcut_command_outcome_from_shell_result(
    request: &ShortcutLaunchRequest,
    shell_result: isize,
) -> ShortcutCommandOutcome {
    shortcut_command_outcome(shortcut_shell_policy_result(request, shell_result))
}

fn shortcut_settings_load_result(
    data_dir: &Path,
) -> Result<crate::store::Settings, ShortcutLaunchResult> {
    match crate::store::load_settings_outcome(data_dir) {
        crate::store::SettingsLoadOutcome::Loaded(settings)
        | crate::store::SettingsLoadOutcome::Missing(settings)
        | crate::store::SettingsLoadOutcome::RecoveredWithDefaults { settings, .. } => Ok(settings),
        crate::store::SettingsLoadOutcome::FatalBackupFailure { error } => {
            Err(shortcut_launch_failure(
                "Shortcut settings unavailable",
                format!("Riftle could not load shortcut settings: {error}"),
            ))
        }
    }
}

fn shell_execute_shortcut(request: &ShortcutLaunchRequest) -> isize {
    let file = to_wide_null(&request.path);
    let parameters = request
        .parameters
        .as_ref()
        .map(|parameters| to_wide_null(parameters));
    let parameter_ptr = parameters
        .as_ref()
        .map(|parameters| parameters.as_ptr())
        .unwrap_or(std::ptr::null());

    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            0,
            std::ptr::null(),
            file.as_ptr(),
            parameter_ptr,
            std::ptr::null(),
            1,
        )
    };

    result as isize
}

fn open_with_dialog(request: &ShortcutLaunchRequest) -> ShortcutLaunchResult {
    let file = to_wide_null(&request.path);
    let info = windows::Win32::UI::Shell::OPENASINFO {
        pcszFile: windows::core::PCWSTR(file.as_ptr()),
        pcszClass: windows::core::PCWSTR::null(),
        oaifInFlags: windows::Win32::UI::Shell::OAIF_EXEC,
    };

    let result = unsafe {
        windows::Win32::UI::Shell::SHOpenWithDialog(
            windows::Win32::Foundation::HWND(std::ptr::null_mut()),
            &info,
        )
    };

    match result {
        Ok(()) => shortcut_launch_success(),
        Err(err) => shortcut_launch_failure(
            "Shortcut launch failed",
            format!(
                "Windows could not choose an application for shortcut target {}: {}.",
                request.path, err
            ),
        ),
    }
}

#[tauri::command]
pub fn launch_shortcut(
    id: String,
    _app: tauri::AppHandle,
    data_dir: tauri::State<PathBuf>,
) -> Result<ShortcutLaunchResult, String> {
    let settings = match shortcut_settings_load_result(data_dir.inner().as_path()) {
        Ok(settings) => settings,
        Err(result) => return Ok(result),
    };
    let Some(request) = resolve_shortcut_from_settings(&settings, &id) else {
        return Ok(shortcut_launch_failure(
            "Shortcut not found",
            format!("Shortcut is no longer configured: {id}"),
        ));
    };

    let target_outcome = shortcut_command_outcome_from_target_policy(&request);
    if !target_outcome.result.success {
        return Ok(target_outcome.result);
    }

    let shell_result = shell_execute_shortcut(&request);
    if !shortcut_shell_failed(shell_result) {
        return Ok(shortcut_command_outcome_from_shell_result(&request, shell_result).result);
    }

    if shortcut_shell_failure_action(&request, shell_result) == ShortcutShellFailureAction::OpenWith
    {
        return Ok(open_with_dialog(&request));
    }

    Ok(shortcut_command_outcome_from_shell_result(&request, shell_result).result)
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
        Err(err)
            if err.kind() == std::io::ErrorKind::NotFound && filename != GENERIC_ICON_FILENAME =>
        {
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
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
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
    fn shortcut_missing_target_policy_returns_visible_failure() {
        let temp = tempdir().unwrap();
        let missing_path = temp.path().join("missing-folder");
        let request = shortcut_launch_request(
            ShortcutTargetKind::Directory,
            missing_path.to_string_lossy().as_ref(),
            "--ignored",
        );

        let result = evaluate_shortcut_target_policy(&request);

        assert!(!result.success);
        let warning = result
            .warning
            .expect("missing target should return warning");
        assert_eq!(warning.kind, "shortcut-launch-failed");
        assert!(warning.message.contains("does not exist"));
    }

    #[test]
    fn shortcut_parameters_only_passed_for_parameter_capable_executables() {
        let exe_request = shortcut_launch_request(
            ShortcutTargetKind::File,
            "C:\\Tools\\cleanup.exe",
            "--all --quiet",
        );
        let document_request = shortcut_launch_request(
            ShortcutTargetKind::File,
            "C:\\Docs\\report.pdf",
            "--ignored",
        );
        let lnk_request = shortcut_launch_request(
            ShortcutTargetKind::File,
            "C:\\Tools\\legacy.lnk",
            "--ignored",
        );

        assert_eq!(exe_request.parameters.as_deref(), Some("--all --quiet"));
        assert_eq!(document_request.parameters, None);
        assert_eq!(lnk_request.parameters, None);
    }

    #[test]
    fn shortcut_open_with_policy_only_for_non_executable_no_association_failures() {
        let doc_request =
            shortcut_launch_request(ShortcutTargetKind::File, "C:\\Docs\\report.unknown", "");
        let exe_request =
            shortcut_launch_request(ShortcutTargetKind::File, "C:\\Tools\\cleanup.exe", "");
        let dir_request =
            shortcut_launch_request(ShortcutTargetKind::Directory, "C:\\Projects", "");

        assert_eq!(
            shortcut_shell_failure_action(&doc_request, SE_ERR_NOASSOC_CODE),
            ShortcutShellFailureAction::OpenWith
        );
        assert_eq!(
            shortcut_shell_failure_action(&exe_request, SE_ERR_NOASSOC_CODE),
            ShortcutShellFailureAction::Warn
        );
        assert_eq!(
            shortcut_shell_failure_action(&dir_request, SE_ERR_NOASSOC_CODE),
            ShortcutShellFailureAction::Warn
        );
        assert_eq!(
            shortcut_shell_failure_action(&doc_request, 2),
            ShortcutShellFailureAction::Warn
        );
    }

    #[test]
    fn shortcut_command_helper_returns_success_for_shell_success_codes() {
        let request =
            shortcut_launch_request(ShortcutTargetKind::File, "C:\\Tools\\cleanup.exe", "");

        let outcome = shortcut_command_outcome_from_shell_result(&request, 33);

        assert!(outcome.result.success);
        assert!(outcome.result.warning.is_none());
        assert!(!outcome.should_hide_launcher);
    }

    #[test]
    fn shortcut_command_helper_returns_warning_for_shell_failure_codes() {
        let request =
            shortcut_launch_request(ShortcutTargetKind::File, "C:\\Tools\\cleanup.exe", "");

        let outcome = shortcut_command_outcome_from_shell_result(&request, 2);

        assert!(!outcome.result.success);
        let warning = outcome
            .result
            .warning
            .expect("failure should return warning");
        assert_eq!(warning.kind, "shortcut-launch-failed");
        assert!(warning.message.contains("ShellExecuteW code 2"));
        assert!(!outcome.should_hide_launcher);
    }

    #[test]
    fn shortcut_command_helper_returns_warning_for_missing_targets_without_hide() {
        let temp = tempdir().unwrap();
        let missing_path = temp.path().join("missing-file.exe");
        let request = shortcut_launch_request(
            ShortcutTargetKind::File,
            missing_path.to_string_lossy().as_ref(),
            "",
        );

        let outcome = shortcut_command_outcome_from_target_policy(&request);

        assert!(!outcome.result.success);
        assert!(outcome.result.warning.is_some());
        assert!(!outcome.should_hide_launcher);
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
