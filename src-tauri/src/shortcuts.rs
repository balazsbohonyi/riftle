use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryShortcut {
    pub path: String,
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileShortcut {
    pub path: String,
    #[serde(default)]
    pub parameters: String,
    #[serde(default)]
    pub alias: String,
}

pub fn shortcut_display_name(path: &str, alias: &str) -> String {
    let trimmed_alias = alias.trim();
    if !trimmed_alias.is_empty() {
        return trimmed_alias.to_string();
    }

    Path::new(path)
        .file_stem()
        .or_else(|| Path::new(path).file_name())
        .map(|name| name.to_string_lossy().trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.trim().to_string())
}

fn normalized_match_name(path: &str, alias: &str) -> String {
    shortcut_display_name(path, alias).to_lowercase()
}

#[allow(dead_code)]
pub fn shortcut_id(kind: &str, path: &str, parameters: &str) -> String {
    let normalized_kind = kind.trim().to_lowercase();
    let normalized_path = path.trim().to_lowercase();
    let normalized_params = parameters.trim().to_lowercase();
    let mut hash = 0xcbf29ce484222325u64;

    for byte in normalized_path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    if !normalized_params.is_empty() {
        for byte in normalized_params.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    format!("shortcut:{normalized_kind}:{hash:016x}")
}

pub fn is_parameterized_executable_target(path: &str) -> bool {
    Path::new(path)
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .map(|extension| matches!(extension.as_str(), "exe" | "com" | "bat" | "cmd"))
        .unwrap_or(false)
}

pub fn validate_shortcuts(
    directory_shortcuts: &[DirectoryShortcut],
    file_shortcuts: &[FileShortcut],
) -> Result<(), String> {
    let mut names = HashSet::new();

    for shortcut in directory_shortcuts {
        if shortcut.path.trim().is_empty() {
            return Err("Directory shortcut path is required.".to_string());
        }

        let name = normalized_match_name(&shortcut.path, &shortcut.alias);
        if name.is_empty() || !names.insert(name.clone()) {
            return Err(format!("duplicate shortcut name: {}", shortcut_display_name(&shortcut.path, &shortcut.alias)));
        }
    }

    for shortcut in file_shortcuts {
        if shortcut.path.trim().is_empty() {
            return Err("File shortcut path is required.".to_string());
        }

        if !shortcut.parameters.trim().is_empty() {
            if !is_parameterized_executable_target(&shortcut.path) {
                return Err(format!(
                    "Shortcut parameters are only supported for .exe, .com, .bat, and .cmd files: {}",
                    shortcut.path
                ));
            }

            if shortcut.alias.trim().is_empty() {
                return Err(format!(
                    "A shortcut alias is required when parameters are set for {}.",
                    shortcut.path
                ));
            }
        }

        let name = normalized_match_name(&shortcut.path, &shortcut.alias);
        if name.is_empty() || !names.insert(name.clone()) {
            return Err(format!("duplicate shortcut name: {}", shortcut_display_name(&shortcut.path, &shortcut.alias)));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(path: &str, alias: &str) -> DirectoryShortcut {
        DirectoryShortcut {
            path: path.to_string(),
            alias: alias.to_string(),
        }
    }

    fn file(path: &str, parameters: &str, alias: &str) -> FileShortcut {
        FileShortcut {
            path: path.to_string(),
            parameters: parameters.to_string(),
            alias: alias.to_string(),
        }
    }

    #[test]
    fn shortcut_duplicate_aliases_or_fallback_names_are_rejected_case_insensitively() {
        let duplicate_alias = validate_shortcuts(
            &[dir("C:\\Tools", "Work")],
            &[file("C:\\Apps\\Editor.exe", "", "work")],
        );
        assert!(duplicate_alias.unwrap_err().contains("duplicate shortcut name"));

        let duplicate_fallback = validate_shortcuts(
            &[dir("C:\\Projects\\Notes", "")],
            &[file("C:\\Docs\\notes.txt", "", "")],
        );
        assert!(duplicate_fallback.unwrap_err().contains("duplicate shortcut name"));
    }

    #[test]
    fn shortcut_parameterized_executable_requires_alias() {
        let result = validate_shortcuts(&[], &[file("C:\\Tools\\cleanup.exe", "--all", "")]);

        assert!(result.unwrap_err().contains("alias"));
    }

    #[test]
    fn shortcut_parameters_are_rejected_for_non_executable_files() {
        let result = validate_shortcuts(&[], &[file("C:\\Docs\\report.pdf", "--all", "Report")]);

        assert!(result.unwrap_err().contains("parameters"));
    }

    #[test]
    fn shortcut_display_name_prefers_alias_then_fallback_filename() {
        assert_eq!(shortcut_display_name("C:\\Tools\\cleanup.exe", "Clean"), "Clean");
        assert_eq!(shortcut_display_name("C:\\Tools\\cleanup.exe", ""), "cleanup");
        assert_eq!(shortcut_display_name("C:\\Projects", ""), "Projects");
    }

    #[test]
    fn shortcut_ids_are_stable_and_kind_prefixed() {
        assert_eq!(
            shortcut_id("dir", "C:\\Projects", ""),
            shortcut_id("dir", "C:\\Projects", "")
        );
        assert!(shortcut_id("dir", "C:\\Projects", "").starts_with("shortcut:dir:"));
        assert!(shortcut_id("file", "C:\\Tools\\cleanup.exe", "").starts_with("shortcut:file:"));
        assert_ne!(
            shortcut_id("dir", "C:\\Projects", ""),
            shortcut_id("file", "C:\\Projects", "")
        );
        // Verify parameters change the ID
        assert_ne!(
            shortcut_id("file", "C:\\Code.exe", "--prj1"),
            shortcut_id("file", "C:\\Code.exe", "--prj2")
        );
    }

    #[test]
    fn shortcut_parameter_policy_only_allows_executable_app_targets() {
        assert!(is_parameterized_executable_target("C:\\Tools\\cleanup.exe"));
        assert!(is_parameterized_executable_target("C:\\Tools\\script.CMD"));
        assert!(!is_parameterized_executable_target("C:\\Tools\\shortcut.lnk"));
        assert!(!is_parameterized_executable_target("C:\\Docs\\report.pdf"));
    }
}
