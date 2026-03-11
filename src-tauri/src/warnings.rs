use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, PoisonError};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendWarning {
    pub kind: String,
    pub title: String,
    pub message: String,
    pub backup_path: Option<String>,
}

#[derive(Clone, Default)]
pub struct PendingBackendWarnings(pub Arc<Mutex<Vec<BackendWarning>>>);

fn lock_pending_warnings(
    pending: &PendingBackendWarnings,
) -> std::sync::MutexGuard<'_, Vec<BackendWarning>> {
    pending
        .0
        .lock()
        .unwrap_or_else(|err: PoisonError<_>| err.into_inner())
}

pub fn enqueue_backend_warning(
    pending: &PendingBackendWarnings,
    warning: BackendWarning,
) {
    lock_pending_warnings(pending).push(warning);
}

pub fn push_backend_warning(app: &AppHandle, warning: BackendWarning) {
    let pending = app.state::<PendingBackendWarnings>();
    enqueue_backend_warning(&pending, warning.clone());

    if let Err(err) = app.emit("backend-warning", warning) {
        eprintln!("[warnings] failed to emit backend-warning: {}", err);
    }
}

#[tauri::command]
pub fn take_backend_warnings(
    pending: tauri::State<PendingBackendWarnings>,
) -> Vec<BackendWarning> {
    std::mem::take(&mut *lock_pending_warnings(&pending))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_warning(kind: &str, index: usize) -> BackendWarning {
        BackendWarning {
            kind: kind.to_string(),
            title: format!("Warning {}", index),
            message: format!("Message {}", index),
            backup_path: Some(format!("C:\\backups\\{}.bak", index)),
        }
    }

    #[test]
    fn backend_warning_push_preserves_order() {
        let pending = PendingBackendWarnings::default();

        enqueue_backend_warning(&pending, sample_warning("db-reset", 1));
        enqueue_backend_warning(&pending, sample_warning("settings-reset", 2));

        let items = lock_pending_warnings(&pending).clone();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], sample_warning("db-reset", 1));
        assert_eq!(items[1], sample_warning("settings-reset", 2));
    }

    #[test]
    fn take_backend_warnings_drains_and_empties_queue() {
        let pending = PendingBackendWarnings::default();
        enqueue_backend_warning(&pending, sample_warning("db-reset", 1));
        enqueue_backend_warning(&pending, sample_warning("settings-reset", 2));

        let drained = std::mem::take(&mut *lock_pending_warnings(&pending));
        assert_eq!(drained.len(), 2);
        assert!(lock_pending_warnings(&pending).is_empty());
    }
}
