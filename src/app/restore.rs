use std::thread;
use std::time::{Duration, Instant};

use crate::domain::error::{DeskshotError, Result};
use crate::domain::model::WindowLayout;
use crate::domain::ports::{
    AppLauncher, MatchOutcome, SnapshotRepository, VirtualDesktops, WindowMatch, WindowMatcher,
    WindowPlacer, WindowSource,
};

/// The collaborators a restore needs, grouped to keep the constructor readable.
pub struct RestorePorts<'a> {
    pub windows: &'a dyn WindowSource,
    pub placer: &'a dyn WindowPlacer,
    pub desktops: &'a dyn VirtualDesktops,
    pub launcher: &'a dyn AppLauncher,
    pub matcher: &'a dyn WindowMatcher,
    pub store: &'a dyn SnapshotRepository,
}

#[derive(Debug, Clone, Copy)]
pub struct RestoreSettings {
    /// How long to wait for launched apps to show their windows.
    pub launch_timeout: Duration,
    pub poll_interval: Duration,
}

impl Default for RestoreSettings {
    fn default() -> Self {
        Self {
            launch_timeout: Duration::from_secs(20),
            poll_interval: Duration::from_millis(500),
        }
    }
}

#[derive(Debug)]
pub struct RestoreFailure {
    pub window: WindowLayout,
    pub error: DeskshotError,
}

#[derive(Debug, Default)]
pub struct RestoreReport {
    pub restored: usize,
    pub launch_failures: Vec<RestoreFailure>,
    pub placement_failures: Vec<RestoreFailure>,
    /// Recorded windows for which no live window appeared in time.
    pub missing: Vec<WindowLayout>,
}

pub struct RestoreService<'a> {
    ports: RestorePorts<'a>,
    settings: RestoreSettings,
}

impl<'a> RestoreService<'a> {
    pub fn new(ports: RestorePorts<'a>, settings: RestoreSettings) -> Self {
        Self { ports, settings }
    }

    /// Loads the snapshot, launches whatever is missing and puts every
    /// window back. Per-window problems are collected in the report rather
    /// than aborting the whole restore.
    pub fn restore(&self) -> Result<RestoreReport> {
        let snapshot = self.ports.store.load()?;
        self.ports.desktops.ensure_count(snapshot.desktop_count)?;

        let mut report = RestoreReport::default();
        let outcome = self.locate_windows(&snapshot.windows, &mut report)?;

        for found in &outcome.matched {
            match self.arrange(found) {
                Ok(()) => report.restored += 1,
                Err(error) => report.placement_failures.push(failure(found.target, error)),
            }
        }
        report.missing = outcome.unmatched.into_iter().cloned().collect();
        Ok(report)
    }

    /// Matches recorded windows to live ones, launching the apps of any
    /// that are missing and waiting for them to appear.
    fn locate_windows<'s>(
        &self,
        targets: &'s [WindowLayout],
        report: &mut RestoreReport,
    ) -> Result<MatchOutcome<'s>> {
        let outcome = self.match_live(targets)?;
        if outcome.unmatched.is_empty() {
            return Ok(outcome);
        }

        self.launch_all(&outcome.unmatched, report);
        let deadline = Instant::now() + self.settings.launch_timeout;
        loop {
            thread::sleep(self.settings.poll_interval);
            let outcome = self.match_live(targets)?;
            if outcome.unmatched.is_empty() || Instant::now() >= deadline {
                return Ok(outcome);
            }
        }
    }

    fn match_live<'s>(&self, targets: &'s [WindowLayout]) -> Result<MatchOutcome<'s>> {
        let live = self.ports.windows.windows()?;
        Ok(self.ports.matcher.pair(targets, &live))
    }

    fn launch_all(&self, missing: &[&WindowLayout], report: &mut RestoreReport) {
        for window in missing {
            if let Err(error) = self.ports.launcher.launch(&window.identity.app) {
                report.launch_failures.push(failure(window, error));
            }
        }
    }

    /// Placement first, then the desktop move: changing show state can
    /// activate a window, and activating one on another desktop would
    /// switch the user's view to it.
    fn arrange(&self, found: &WindowMatch) -> Result<()> {
        self.ports.placer.apply(found.window, &found.target.placement)?;
        if let Some(desktop) = found.target.desktop {
            self.ports.desktops.move_window(found.window, desktop)?;
        }
        Ok(())
    }
}

fn failure(window: &WindowLayout, error: DeskshotError) -> RestoreFailure {
    RestoreFailure { window: window.clone(), error }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fakes::*;
    use crate::app::matching::ExecutableMatcher;
    use crate::domain::model::{Snapshot, WindowHandle};

    const INSTANT: RestoreSettings = RestoreSettings {
        launch_timeout: Duration::ZERO,
        poll_interval: Duration::ZERO,
    };

    struct Fixture {
        windows: FakeWindowSource,
        placer: FakePlacer,
        desktops: FakeDesktops,
        launcher: FakeLauncher,
        store: FakeStore,
    }

    impl Fixture {
        fn new(snapshot: Snapshot, live: Vec<crate::domain::model::LiveWindow>) -> Self {
            Self {
                windows: FakeWindowSource::new(live),
                placer: FakePlacer::default(),
                desktops: FakeDesktops::with_count(1),
                launcher: FakeLauncher::default(),
                store: FakeStore::containing(snapshot),
            }
        }

        fn restore(&self) -> RestoreReport {
            let ports = RestorePorts {
                windows: &self.windows,
                placer: &self.placer,
                desktops: &self.desktops,
                launcher: &self.launcher,
                matcher: &ExecutableMatcher,
                store: &self.store,
            };
            RestoreService::new(ports, INSTANT).restore().unwrap()
        }
    }

    #[test]
    fn places_open_windows_and_moves_them_to_their_desktop() {
        let mut target = layout("editor.exe", "notes");
        target.desktop = Some(2);
        let fixture = Fixture::new(
            Snapshot { desktop_count: 3, windows: vec![target.clone()] },
            vec![live(5, "editor.exe", "notes")],
        );

        let report = fixture.restore();

        let handle = WindowHandle::from_raw(5);
        assert_eq!(report.restored, 1);
        assert_eq!(fixture.desktops.count().unwrap(), 3);
        assert_eq!(fixture.placer.applied(), vec![(handle, target.placement)]);
        assert_eq!(fixture.desktops.moves(), vec![(handle, 2)]);
        assert!(fixture.launcher.launched().is_empty());
    }

    #[test]
    fn launches_missing_apps_and_reports_those_that_never_appear() {
        let fixture = Fixture::new(
            Snapshot { desktop_count: 1, windows: vec![layout("browser.exe", "News")] },
            vec![],
        );

        let report = fixture.restore();

        assert_eq!(fixture.launcher.launched(), vec!["browser.exe".to_string()]);
        assert_eq!(report.restored, 0);
        assert_eq!(report.missing.len(), 1);
    }
}
