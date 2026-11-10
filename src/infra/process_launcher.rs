use std::process::Command;

use crate::domain::error::{DeskshotError, Result};
use crate::domain::model::AppId;
use crate::domain::ports::AppLauncher;

/// Starts the executable directly, with its own folder as working directory.
pub struct ProcessLauncher;

impl AppLauncher for ProcessLauncher {
    fn launch(&self, app: &AppId) -> Result<()> {
        let executable = app.executable();
        let mut command = Command::new(executable);
        if let Some(folder) = executable.parent() {
            command.current_dir(folder);
        }
        command.spawn().map(drop).map_err(|e| DeskshotError::Launch {
            app: app.to_string(),
            reason: e.to_string(),
        })
    }
}
