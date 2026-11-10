use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_SHOWMAXIMIZED, SW_SHOWMINNOACTIVE, SW_SHOWNOACTIVATE,
    WPF_ASYNCWINDOWPLACEMENT,
};

use super::{sys, to_hwnd};
use crate::domain::error::Result;
use crate::domain::model::{Placement, Rect, ShowState, WindowHandle};
use crate::domain::ports::WindowPlacer;

/// Restores state and restored size with Get/SetWindowPlacement, then, for
/// normal windows, the exact on-screen frame. The second step is what
/// brings snapped windows back: Windows reports their pre-snap size as the
/// restored rectangle.
pub struct Win32WindowPlacer;

impl WindowPlacer for Win32WindowPlacer {
    fn apply(&self, window: WindowHandle, placement: &Placement) -> Result<()> {
        let hwnd = to_hwnd(window);
        let mut native = sys::get_placement(hwnd)?;
        native.rcNormalPosition = to_native_rect(placement.bounds);
        native.showCmd = show_command(placement.state).0 as u32;
        // Don't block if the target window's thread is busy or hung.
        native.flags = WPF_ASYNCWINDOWPLACEMENT;
        sys::set_placement(hwnd, &native)?;

        match (placement.state, placement.frame) {
            (ShowState::Normal, Some(frame)) => sys::set_window_rect(hwnd, to_native_rect(frame)),
            _ => Ok(()),
        }
    }
}

pub(super) fn read(hwnd: HWND) -> Result<Placement> {
    let native = sys::get_placement(hwnd)?;
    let state = current_state(hwnd);
    let frame = match state {
        ShowState::Normal => Some(from_native_rect(sys::window_rect(hwnd)?)),
        ShowState::Minimized | ShowState::Maximized => None,
    };
    Ok(Placement { bounds: from_native_rect(native.rcNormalPosition), state, frame })
}

fn current_state(hwnd: HWND) -> ShowState {
    if sys::is_minimized(hwnd) {
        ShowState::Minimized
    } else if sys::is_maximized(hwnd) {
        ShowState::Maximized
    } else {
        ShowState::Normal
    }
}

fn show_command(state: ShowState) -> SHOW_WINDOW_CMD {
    match state {
        ShowState::Normal => SW_SHOWNOACTIVATE,
        ShowState::Minimized => SW_SHOWMINNOACTIVE,
        ShowState::Maximized => SW_SHOWMAXIMIZED,
    }
}

fn to_native_rect(rect: Rect) -> RECT {
    RECT { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }
}

fn from_native_rect(rect: RECT) -> Rect {
    Rect { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }
}
