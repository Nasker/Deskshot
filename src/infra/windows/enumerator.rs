use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;

use super::{placement, sys, to_handle};
use crate::domain::error::Result;
use crate::domain::model::{AppId, LiveWindow, WindowIdentity};
use crate::domain::ports::WindowSource;

/// Shell surfaces, plus UWP frames whose owning process
/// (ApplicationFrameHost) can't be relaunched directly.
const IGNORED_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "ApplicationFrameWindow",
    "Windows.UI.Core.CoreWindow",
];

/// Enumerates the windows that show up in the taskbar / Alt+Tab, except
/// deskshot's own.
pub struct Win32WindowSource;

impl Win32WindowSource {
    fn is_restorable(hwnd: HWND, class_name: &str) -> bool {
        sys::process_id(hwnd) != std::process::id()
            && sys::is_visible(hwnd)
            && !sys::has_owner(hwnd)
            && (sys::extended_style(hwnd) & WS_EX_TOOLWINDOW).0 == 0
            && !sys::is_cloaked_by_app(hwnd)
            && !IGNORED_CLASSES.contains(&class_name)
    }

    /// `None` for windows we can't fully describe, e.g. those owned by
    /// protected processes we aren't allowed to query.
    fn describe(hwnd: HWND, class_name: String) -> Option<LiveWindow> {
        let title = sys::window_title(hwnd);
        if title.is_empty() {
            return None;
        }
        let app = AppId::new(sys::process_image_path(sys::process_id(hwnd)).ok()?);
        Some(LiveWindow {
            handle: to_handle(hwnd),
            identity: WindowIdentity { app, title, class_name },
            placement: placement::read(hwnd).ok()?,
        })
    }
}

impl WindowSource for Win32WindowSource {
    fn windows(&self) -> Result<Vec<LiveWindow>> {
        Ok(sys::top_level_windows()?
            .into_iter()
            .filter_map(|hwnd| {
                let class_name = sys::class_name(hwnd);
                Self::is_restorable(hwnd, &class_name)
                    .then(|| Self::describe(hwnd, class_name))
                    .flatten()
            })
            .collect())
    }
}
