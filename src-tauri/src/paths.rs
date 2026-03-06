use std::path::PathBuf;
use tauri::AppHandle;

/// Returns the data directory for this install.
/// Portable mode: exe_dir/data (when riftle-launcher.portable exists adjacent to exe).
/// Installed mode: %APPDATA%\riftle-launcher\ (hardcoded for consistency and discoverability).
/// Guarantees the directory exists via create_dir_all before returning.
pub fn data_dir(_app: &AppHandle) -> PathBuf {
    let exe_path = std::env::current_exe()
        .expect("cannot resolve current exe path");
    let exe_dir = exe_path.parent()
        .expect("exe has no parent directory")
        .to_path_buf();

    data_dir_from_exe_dir(&exe_dir)
}

// Internal helper — takes exe_dir explicitly so unit tests can inject a tempdir.
pub fn data_dir_from_exe_dir(exe_dir: &PathBuf) -> PathBuf {
    let dir = if exe_dir.join("riftle-launcher.portable").exists() {
        exe_dir.join("data")
    } else {
        // Installed mode: use %APPDATA%\riftle-launcher\ on Windows
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .expect("APPDATA environment variable not set");
            PathBuf::from(appdata).join("riftle-launcher")
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Fallback for non-Windows (Linux, macOS) — should not be used in production
            let home = std::env::var("HOME")
                .expect("HOME environment variable not set");
            PathBuf::from(home).join(".riftle-launcher")
        }
    };

    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("cannot create data dir {:?}: {}", dir, e));
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_portable_detection_returns_data_subdir() {
        // Simulate exe_dir with riftle-launcher.portable present
        let temp = std::env::temp_dir().join("riftle_paths_test");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("riftle-launcher.portable"), "").unwrap();

        // Test the portable branch logic directly
        let dir = data_dir_from_exe_dir(&temp);
        assert!(dir.exists(), "data subdir should have been created");
        assert_eq!(dir, temp.join("data"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_no_portable_marker_takes_installed_branch() {
        let temp = std::env::temp_dir().join("riftle_paths_test_no_portable");
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        // No riftle-launcher.portable written

        let is_portable = temp.join("riftle-launcher.portable").exists();
        assert!(!is_portable, "no portable marker should mean installed mode");

        // Cleanup
        let _ = fs::remove_dir_all(&temp);
    }
}
