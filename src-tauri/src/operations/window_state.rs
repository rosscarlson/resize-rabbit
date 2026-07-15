use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::api::path;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, Window};

use crate::debug_log;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn get_window_state_path<R: Runtime>(app_handle: &AppHandle<R>) -> Option<PathBuf> {
    let dir = path::app_data_dir(&app_handle.config())?;

    if !dir.exists() {
        fs::create_dir_all(&dir).ok()?;
    }

    Some(dir.join("window_state.json"))
}

/// Saves the main window's current monitor/position/size so it can reopen in the same
/// spot next launch. Windows always spawns a window with no explicit x/y on the primary
/// monitor, which is why the app previously always reappeared on monitor 1 for
/// multi-monitor users.
pub fn save_window_state<R: Runtime>(window: &Window<R>) {
    let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return;
    };

    let state = WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    };

    let Some(path) = get_window_state_path(&window.app_handle()) else {
        return;
    };

    match serde_json::to_string_pretty(&state) {
        Ok(json) => {
            if let Err(e) = fs::write(path, json) {
                debug_log!("Failed to save window state: {}", e);
            }
        }
        Err(e) => debug_log!("Failed to serialize window state: {}", e),
    }
}

fn load_window_state<R: Runtime>(app_handle: &AppHandle<R>) -> Option<WindowState> {
    let path = get_window_state_path(app_handle)?;
    let json = fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

/// Only restores a saved position if it still overlaps a currently connected monitor,
/// so a saved position from a monitor that's since been unplugged doesn't strand the
/// window off-screen where the user can't get to it.
fn fits_on_a_monitor<R: Runtime>(window: &Window<R>, state: &WindowState) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        return false;
    };

    let (win_left, win_top) = (state.x, state.y);
    let (win_right, win_bottom) = (state.x + state.width as i32, state.y + state.height as i32);

    monitors.iter().any(|monitor| {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let (mon_left, mon_top) = (m_pos.x, m_pos.y);
        let (mon_right, mon_bottom) = (m_pos.x + m_size.width as i32, m_pos.y + m_size.height as i32);

        win_left < mon_right && win_right > mon_left && win_top < mon_bottom && win_bottom > mon_top
    })
}

pub fn restore_window_state<R: Runtime>(window: &Window<R>) {
    let Some(state) = load_window_state(&window.app_handle()) else {
        return;
    };

    if !fits_on_a_monitor(window, &state) {
        debug_log!("Saved window position no longer fits a connected monitor, ignoring");
        return;
    }

    let _ = window.set_size(PhysicalSize::new(state.width, state.height));
    let _ = window.set_position(PhysicalPosition::new(state.x, state.y));
}
