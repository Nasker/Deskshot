//! Composition root: the only place that knows which concrete adapters
//! implement the ports. Front-ends call these two functions and nothing else.

use std::path::Path;

use crate::app::restore::RestoreReport;
use crate::domain::error::Result;
use crate::domain::model::Snapshot;

#[cfg(windows)]
pub fn capture(path: &Path) -> Result<Snapshot> {
    use crate::app::capture::CaptureService;
    use crate::infra::json_store::JsonFileStore;
    use crate::infra::windows::{init_process, Win32WindowSource, WinVdDesktops};

    init_process();
    let store = JsonFileStore::new(path);
    CaptureService::new(&Win32WindowSource, &WinVdDesktops, &store).capture()
}

#[cfg(windows)]
pub fn restore(path: &Path) -> Result<RestoreReport> {
    use crate::app::matching::ExecutableMatcher;
    use crate::app::restore::{RestorePorts, RestoreService, RestoreSettings};
    use crate::infra::json_store::JsonFileStore;
    use crate::infra::process_launcher::ProcessLauncher;
    use crate::infra::windows::{init_process, Win32WindowPlacer, Win32WindowSource, WinVdDesktops};

    init_process();
    let store = JsonFileStore::new(path);
    let ports = RestorePorts {
        windows: &Win32WindowSource,
        placer: &Win32WindowPlacer,
        desktops: &WinVdDesktops,
        launcher: &ProcessLauncher,
        matcher: &ExecutableMatcher,
        store: &store,
    };
    RestoreService::new(ports, RestoreSettings::default()).restore()
}

#[cfg(not(windows))]
pub fn capture(_path: &Path) -> Result<Snapshot> {
    Err(crate::domain::error::DeskshotError::Unsupported("capture"))
}

#[cfg(not(windows))]
pub fn restore(_path: &Path) -> Result<RestoreReport> {
    Err(crate::domain::error::DeskshotError::Unsupported("restore"))
}
