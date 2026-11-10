use crate::domain::error::Result;
use crate::domain::model::{LiveWindow, Snapshot, WindowLayout};
use crate::domain::ports::{SnapshotRepository, VirtualDesktops, WindowSource};

pub struct CaptureService<'a> {
    windows: &'a dyn WindowSource,
    desktops: &'a dyn VirtualDesktops,
    store: &'a dyn SnapshotRepository,
}

impl<'a> CaptureService<'a> {
    pub fn new(
        windows: &'a dyn WindowSource,
        desktops: &'a dyn VirtualDesktops,
        store: &'a dyn SnapshotRepository,
    ) -> Self {
        Self { windows, desktops, store }
    }

    /// Records every restorable window and persists the result.
    pub fn capture(&self) -> Result<Snapshot> {
        let windows = self
            .windows
            .windows()?
            .into_iter()
            .map(|window| self.to_layout(window))
            .collect::<Result<Vec<_>>>()?;

        let snapshot = Snapshot {
            desktop_count: self.desktops.count()?,
            windows,
        };
        self.store.save(&snapshot)?;
        Ok(snapshot)
    }

    fn to_layout(&self, window: LiveWindow) -> Result<WindowLayout> {
        Ok(WindowLayout {
            desktop: self.desktops.desktop_of(window.handle)?,
            identity: window.identity,
            placement: window.placement,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fakes::*;

    #[test]
    fn records_windows_with_their_desktops_and_saves() {
        let editor = live(1, "editor.exe", "notes.txt");
        let browser = live(2, "browser.exe", "News");
        let windows = FakeWindowSource::new(vec![editor.clone(), browser.clone()]);
        let desktops = FakeDesktops::with_count(3).with_window_on(browser.handle, 2);
        let store = FakeStore::default();

        let snapshot = CaptureService::new(&windows, &desktops, &store).capture().unwrap();

        assert_eq!(snapshot.desktop_count, 3);
        assert_eq!(snapshot.windows[0].desktop, None);
        assert_eq!(snapshot.windows[1].desktop, Some(2));
        assert_eq!(snapshot.windows[1].identity, browser.identity);
        assert_eq!(store.saved(), Some(snapshot));
    }
}
