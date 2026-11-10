//! Thin safe wrappers over the Win32 calls this crate needs. This is the
//! only module containing `unsafe`.

use std::ffi::c_void;
use std::path::PathBuf;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWM_CLOAKED_APP};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowLongW, GetWindowPlacement, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
    SetWindowPlacement, SetWindowPos, GWL_EXSTYLE, GW_OWNER, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
    SWP_NOZORDER, WINDOWPLACEMENT, WINDOW_EX_STYLE,
};

use crate::domain::error::{DeskshotError, Result};

fn platform_error(error: windows::core::Error) -> DeskshotError {
    DeskshotError::Platform(error.to_string())
}

pub fn enable_per_monitor_dpi_awareness() {
    // Fails harmlessly if awareness was already set (e.g. by a manifest).
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

pub fn top_level_windows() -> Result<Vec<HWND>> {
    unsafe extern "system" fn collect(hwnd: HWND, state: LPARAM) -> BOOL {
        let windows = unsafe { &mut *(state.0 as *mut Vec<HWND>) };
        windows.push(hwnd);
        TRUE
    }

    let mut windows: Vec<HWND> = Vec::new();
    unsafe { EnumWindows(Some(collect), LPARAM(&mut windows as *mut Vec<HWND> as isize)) }
        .map_err(platform_error)?;
    Ok(windows)
}

pub fn is_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

pub fn has_owner(hwnd: HWND) -> bool {
    unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok_and(|owner| !owner.is_invalid())
}

pub fn extended_style(hwnd: HWND) -> WINDOW_EX_STYLE {
    WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32)
}

/// True when the app itself has hidden the window via DWM cloaking (typical
/// of suspended or background UWP frames). Windows on *other* virtual
/// desktops are cloaked by the shell instead and are deliberately kept.
pub fn is_cloaked_by_app(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    let queried = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut c_void,
            size_of::<u32>() as u32,
        )
    };
    queried.is_ok() && cloaked & DWM_CLOAKED_APP != 0
}

pub fn window_title(hwnd: HWND) -> String {
    let mut buffer = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    utf16_prefix(&buffer, len)
}

pub fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    utf16_prefix(&buffer, len)
}

pub fn process_id(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

pub fn process_image_path(pid: u32) -> Result<PathBuf> {
    let process = OwnedHandle(
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
            .map_err(platform_error)?,
    );
    let mut buffer = [0u16; 1024];
    let mut len = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(process.0, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut len)
    }
    .map_err(platform_error)?;
    Ok(PathBuf::from(String::from_utf16_lossy(&buffer[..len as usize])))
}

pub fn is_minimized(hwnd: HWND) -> bool {
    unsafe { IsIconic(hwnd) }.as_bool()
}

pub fn is_maximized(hwnd: HWND) -> bool {
    unsafe { IsZoomed(hwnd) }.as_bool()
}

pub fn get_placement(hwnd: HWND) -> Result<WINDOWPLACEMENT> {
    let mut placement = WINDOWPLACEMENT {
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowPlacement(hwnd, &mut placement) }.map_err(platform_error)?;
    Ok(placement)
}

pub fn set_placement(hwnd: HWND, placement: &WINDOWPLACEMENT) -> Result<()> {
    unsafe { SetWindowPlacement(hwnd, placement) }.map_err(platform_error)
}

/// Outer window rectangle in screen coordinates.
pub fn window_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.map_err(platform_error)?;
    Ok(rect)
}

/// Moves and resizes without activating or changing z-order. Async so it
/// queues behind an async `set_placement` on the window's own thread.
pub fn set_window_rect(hwnd: HWND, rect: RECT) -> Result<()> {
    unsafe {
        SetWindowPos(
            hwnd,
            HWND::default(),
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
        )
    }
    .map_err(platform_error)
}

fn utf16_prefix(buffer: &[u16], len: i32) -> String {
    let len = usize::try_from(len).unwrap_or(0).min(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

/// Closes a process handle when dropped.
struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
