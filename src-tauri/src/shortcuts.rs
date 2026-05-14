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
