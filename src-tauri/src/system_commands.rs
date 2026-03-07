// Phase 6: System commands — lock, shutdown, restart, sleep

use tauri::Manager;

#[tauri::command]
pub fn run_system_command(cmd: String, app: tauri::AppHandle) -> Result<(), String> {
    // Hide launcher window before all system commands (irreversible actions — hide first)
    if let Some(win) = app.get_webview_window("launcher") {
        let _ = win.hide();
    }

    // System command IDs from the search index are prefixed with "system:" (e.g. "system:lock").
    // Strip the prefix so match arms stay clean.
    let cmd_key = cmd.strip_prefix("system:").unwrap_or(cmd.as_str());

    match cmd_key {
        "lock" => {
            let result = unsafe { windows_sys::Win32::System::Shutdown::LockWorkStation() };
            if result == 0 {
                eprintln!("[system_command] LockWorkStation failed");
            }
        }
        "shutdown" => {
            let _ = std::process::Command::new("shutdown")
                .args(["/s", "/t", "0"])
                .spawn();
        }
        "restart" => {
            let _ = std::process::Command::new("shutdown")
                .args(["/r", "/t", "0"])
                .spawn();
        }
        "sleep" => {
            // bHibernate=false, bForce=false, bWakeupEventsDisabled=false
            unsafe { windows_sys::Win32::System::Power::SetSuspendState(0, 0, 0); }
        }
        _ => {
            eprintln!("[system_command] unknown command: {}", cmd);
        }
    }

    Ok(())
}
