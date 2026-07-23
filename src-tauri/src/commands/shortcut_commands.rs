use tauri::{AppHandle, GlobalShortcutManager, Runtime};

use crate::setup::shortcuts;

/// Temporarily unregisters every global shortcut so the shortcut-capture UI
/// can read a raw keydown for a combo that's already bound to another
/// profile. Windows delivers a registered global hotkey as `WM_HOTKEY`
/// instead of a normal keystroke — the webview's own keydown listener never
/// sees it at all while that exact combo is still registered, which is why a
/// second (or third) profile could never capture a shortcut already in use.
/// Pair with `shortcuts_resume` once capture ends (success, Escape, or
/// clicking away).
#[tauri::command]
pub fn shortcuts_suspend<R: Runtime>(app_handle: AppHandle<R>) {
    let _ = app_handle.global_shortcut_manager().unregister_all();
}

#[tauri::command]
pub fn shortcuts_resume<R: Runtime>(app_handle: AppHandle<R>) {
    shortcuts::rebuild_shortcuts(&app_handle);
}
