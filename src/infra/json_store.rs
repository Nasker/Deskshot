use std::fs;
use std::path::PathBuf;

use crate::domain::error::{DeskshotError, Result};
use crate::domain::model::Snapshot;
use crate::domain::ports::SnapshotRepository;

/// Stores a snapshot as pretty-printed JSON, so it is easy to inspect and
/// hand-edit.
pub struct JsonFileStore {
    path: PathBuf,
}

impl JsonFileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn storage_error(&self, error: impl std::fmt::Display) -> DeskshotError {
        DeskshotError::Storage(format!("{}: {error}", self.path.display()))
    }
}

impl SnapshotRepository for JsonFileStore {
    fn save(&self, snapshot: &Snapshot) -> Result<()> {
        let json = serde_json::to_string_pretty(snapshot).map_err(|e| self.storage_error(e))?;
        fs::write(&self.path, json).map_err(|e| self.storage_error(e))
    }

    fn load(&self) -> Result<Snapshot> {
        let json = fs::read_to_string(&self.path).map_err(|e| self.storage_error(e))?;
        serde_json::from_str(&json).map_err(|e| self.storage_error(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fakes::layout;

    #[test]
    fn round_trips_a_snapshot() {
        let path = std::env::temp_dir().join(format!("deskshot-test-{}.json", std::process::id()));
        let store = JsonFileStore::new(&path);
        let snapshot = Snapshot { desktop_count: 2, windows: vec![layout("editor.exe", "notes")] };

        store.save(&snapshot).unwrap();
        let loaded = store.load();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.unwrap(), snapshot);
    }

    #[test]
    fn loads_snapshots_saved_before_frame_existed() {
        let json = r#"{"desktop_count":1,"windows":[{"identity":{"app":"a.exe","title":"t","class_name":"c"},
            "desktop":0,"placement":{"bounds":{"left":0,"top":0,"right":1,"bottom":1},"state":"normal"}}]}"#;

        let snapshot: Snapshot = serde_json::from_str(json).unwrap();

        assert_eq!(snapshot.windows[0].placement.frame, None);
    }
}
