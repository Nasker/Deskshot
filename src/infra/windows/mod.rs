//! Win32 adapters. All `unsafe` FFI lives in `sys`; the other modules
//! translate between Win32 types and the domain model.

mod desktops;
mod enumerator;
mod placement;
mod sys;

pub use desktops::WinVdDesktops;
pub use enumerator::Win32WindowSource;
pub use placement::Win32WindowPlacer;

use crate::domain::model::WindowHandle;
use ::windows::Win32::Foundation::HWND;

/// Makes window coordinates physical pixels on every monitor, so captured
/// and restored rectangles mean the same thing across mixed-DPI setups.
pub fn init_process() {
    sys::enable_per_monitor_dpi_awareness();
}

fn to_hwnd(handle: WindowHandle) -> HWND {
    HWND(handle.raw() as *mut core::ffi::c_void)
}

fn to_handle(hwnd: HWND) -> WindowHandle {
    WindowHandle::from_raw(hwnd.0 as isize)
}
