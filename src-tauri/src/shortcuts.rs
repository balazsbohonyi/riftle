use serde::{Deserialize, Serialize};

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
            shortcut_id("dir", "C:\\Projects"),
            shortcut_id("dir", "C:\\Projects")
        );
        assert!(shortcut_id("dir", "C:\\Projects").starts_with("shortcut:dir:"));
        assert!(shortcut_id("file", "C:\\Tools\\cleanup.exe").starts_with("shortcut:file:"));
        assert_ne!(
            shortcut_id("dir", "C:\\Projects"),
            shortcut_id("file", "C:\\Projects")
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
