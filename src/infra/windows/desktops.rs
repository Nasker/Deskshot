use super::to_hwnd;
use crate::domain::error::{DeskshotError, Result};
use crate::domain::model::WindowHandle;
use crate::domain::ports::VirtualDesktops;

/// Virtual desktops through the `winvd` crate, which wraps Windows'
/// undocumented desktop COM interfaces. Those change between Windows
/// builds, so keeping `winvd` up to date is the first fix if this breaks.
pub struct WinVdDesktops;

fn desktop_error(error: winvd::Error) -> DeskshotError {
    DeskshotError::VirtualDesktop(format!("{error:?}"))
}

impl VirtualDesktops for WinVdDesktops {
    fn count(&self) -> Result<u32> {
        winvd::get_desktop_count().map_err(desktop_error)
    }

    fn ensure_count(&self, count: u32) -> Result<()> {
        while self.count()? < count {
            winvd::create_desktop().map_err(desktop_error)?;
        }
        Ok(())
    }

    fn desktop_of(&self, window: WindowHandle) -> Result<Option<u32>> {
        let hwnd = to_hwnd(window);
        if winvd::is_pinned_window(hwnd).unwrap_or(false) {
            return Ok(None);
        }
        match winvd::get_desktop_by_window(hwnd) {
            Ok(desktop) => desktop.get_index().map(Some).map_err(desktop_error),
            Err(winvd::Error::WindowNotFound | winvd::Error::ComElementNotFound) => Ok(None),
            Err(error) => Err(desktop_error(error)),
        }
    }

    fn move_window(&self, window: WindowHandle, desktop: u32) -> Result<()> {
        winvd::move_window_to_desktop(desktop, &to_hwnd(window)).map_err(desktop_error)
    }
}
