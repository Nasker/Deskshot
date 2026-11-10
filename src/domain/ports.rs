//! The interfaces the application layer is written against. Each trait
//! covers one capability so adapters (and test fakes) stay small.

use super::error::Result;
use super::model::{AppId, LiveWindow, Placement, Snapshot, WindowHandle, WindowLayout};

/// Lists the top-level application windows that are worth restoring.
pub trait WindowSource {
    fn windows(&self) -> Result<Vec<LiveWindow>>;
}

/// Moves, resizes and min/maximizes a window.
pub trait WindowPlacer {
    fn apply(&self, window: WindowHandle, placement: &Placement) -> Result<()>;
}

/// Virtual desktop operations. Indices are zero-based.
pub trait VirtualDesktops {
    fn count(&self) -> Result<u32>;
    fn ensure_count(&self, count: u32) -> Result<()>;
    fn desktop_of(&self, window: WindowHandle) -> Result<Option<u32>>;
    fn move_window(&self, window: WindowHandle, desktop: u32) -> Result<()>;
}

/// Starts an application.
pub trait AppLauncher {
    fn launch(&self, app: &AppId) -> Result<()>;
}

/// Persists snapshots.
pub trait SnapshotRepository {
    fn save(&self, snapshot: &Snapshot) -> Result<()>;
    fn load(&self) -> Result<Snapshot>;
}

/// A recorded layout paired with the live window that should take it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowMatch<'a> {
    pub target: &'a WindowLayout,
    pub window: WindowHandle,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MatchOutcome<'a> {
    pub matched: Vec<WindowMatch<'a>>,
    pub unmatched: Vec<&'a WindowLayout>,
}

/// Decides which live window corresponds to which recorded layout.
/// Each live window is assigned to at most one target.
pub trait WindowMatcher {
    fn pair<'a>(&self, targets: &'a [WindowLayout], live: &[LiveWindow]) -> MatchOutcome<'a>;
}
