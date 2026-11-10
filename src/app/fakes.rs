//! In-memory implementations of the ports, for unit tests only.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::domain::error::{DeskshotError, Result};
use crate::domain::model::*;
use crate::domain::ports::*;

pub fn identity(exe: &str, title: &str) -> WindowIdentity {
    WindowIdentity { app: AppId::new(exe), title: title.into(), class_name: "Main".into() }
}

pub fn placement() -> Placement {
    Placement {
        bounds: Rect { left: 10, top: 20, right: 810, bottom: 620 },
        state: ShowState::Normal,
        frame: None,
    }
}

pub fn live(raw: isize, exe: &str, title: &str) -> LiveWindow {
    LiveWindow { handle: WindowHandle::from_raw(raw), identity: identity(exe, title), placement: placement() }
}

pub fn layout(exe: &str, title: &str) -> WindowLayout {
    WindowLayout { identity: identity(exe, title), desktop: None, placement: placement() }
}

pub struct FakeWindowSource(Vec<LiveWindow>);

impl FakeWindowSource {
    pub fn new(windows: Vec<LiveWindow>) -> Self {
        Self(windows)
    }
}

impl WindowSource for FakeWindowSource {
    fn windows(&self) -> Result<Vec<LiveWindow>> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
pub struct FakePlacer(RefCell<Vec<(WindowHandle, Placement)>>);

impl FakePlacer {
    pub fn applied(&self) -> Vec<(WindowHandle, Placement)> {
        self.0.borrow().clone()
    }
}

impl WindowPlacer for FakePlacer {
    fn apply(&self, window: WindowHandle, placement: &Placement) -> Result<()> {
        self.0.borrow_mut().push((window, *placement));
        Ok(())
    }
}

pub struct FakeDesktops {
    count: RefCell<u32>,
    assignments: HashMap<WindowHandle, u32>,
    moves: RefCell<Vec<(WindowHandle, u32)>>,
}

impl FakeDesktops {
    pub fn with_count(count: u32) -> Self {
        Self { count: RefCell::new(count), assignments: HashMap::new(), moves: RefCell::default() }
    }

    pub fn with_window_on(mut self, window: WindowHandle, desktop: u32) -> Self {
        self.assignments.insert(window, desktop);
        self
    }

    pub fn moves(&self) -> Vec<(WindowHandle, u32)> {
        self.moves.borrow().clone()
    }
}

impl VirtualDesktops for FakeDesktops {
    fn count(&self) -> Result<u32> {
        Ok(*self.count.borrow())
    }

    fn ensure_count(&self, count: u32) -> Result<()> {
        let mut current = self.count.borrow_mut();
        *current = (*current).max(count);
        Ok(())
    }

    fn desktop_of(&self, window: WindowHandle) -> Result<Option<u32>> {
        Ok(self.assignments.get(&window).copied())
    }

    fn move_window(&self, window: WindowHandle, desktop: u32) -> Result<()> {
        self.moves.borrow_mut().push((window, desktop));
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeLauncher(RefCell<Vec<String>>);

impl FakeLauncher {
    pub fn launched(&self) -> Vec<String> {
        self.0.borrow().clone()
    }
}

impl AppLauncher for FakeLauncher {
    fn launch(&self, app: &AppId) -> Result<()> {
        self.0.borrow_mut().push(app.to_string());
        Ok(())
    }
}

#[derive(Default)]
pub struct FakeStore(RefCell<Option<Snapshot>>);

impl FakeStore {
    pub fn containing(snapshot: Snapshot) -> Self {
        Self(RefCell::new(Some(snapshot)))
    }

    pub fn saved(&self) -> Option<Snapshot> {
        self.0.borrow().clone()
    }
}

impl SnapshotRepository for FakeStore {
    fn save(&self, snapshot: &Snapshot) -> Result<()> {
        *self.0.borrow_mut() = Some(snapshot.clone());
        Ok(())
    }

    fn load(&self) -> Result<Snapshot> {
        self.saved().ok_or_else(|| DeskshotError::Storage("empty".into()))
    }
}
