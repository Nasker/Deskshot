use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Identifies an application by the executable that owns its windows.
///
/// Equality is case-insensitive because Windows paths are.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(PathBuf);

impl AppId {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self(executable.into())
    }

    pub fn executable(&self) -> &Path {
        &self.0
    }

    fn normalized(&self) -> String {
        self.0.to_string_lossy().to_lowercase()
    }
}

impl PartialEq for AppId {
    fn eq(&self, other: &Self) -> bool {
        self.normalized() == other.normalized()
    }
}

impl Eq for AppId {}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// A rectangle in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowState {
    Normal,
    Minimized,
    Maximized,
}

/// Where a window lives on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placement {
    /// The *restored* rectangle, as the platform reports it. A maximized
    /// window keeps the size it un-maximizes to and the monitor it
    /// maximizes on; a snapped window keeps its size from before the snap.
    pub bounds: Rect,
    pub state: ShowState,
    /// The rectangle actually on screen in normal state, in screen
    /// coordinates. Differs from `bounds` for snapped/tiled windows.
    /// Optional so snapshots taken before it existed still load.
    #[serde(default)]
    pub frame: Option<Rect>,
}

/// What a window *is*, as opposed to where it is. Used to recognise
/// windows again after a relaunch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowIdentity {
    pub app: AppId,
    pub title: String,
    pub class_name: String,
}

/// Opaque reference to a window that currently exists. Only the platform
/// adapters know how to interpret the raw value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowHandle(isize);

impl WindowHandle {
    pub fn from_raw(raw: isize) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> isize {
        self.0
    }
}

/// A window as observed right now.
#[derive(Debug, Clone)]
pub struct LiveWindow {
    pub handle: WindowHandle,
    pub identity: WindowIdentity,
    pub placement: Placement,
}

/// A window as recorded in a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowLayout {
    pub identity: WindowIdentity,
    /// Zero-based virtual desktop index; `None` for pinned or untracked windows.
    pub desktop: Option<u32>,
    pub placement: Placement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub desktop_count: u32,
    pub windows: Vec<WindowLayout>,
}
