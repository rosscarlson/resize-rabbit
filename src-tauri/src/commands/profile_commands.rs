use crate::errors::profile::Error as ProfileError;
use crate::errors::window_manager::Error as WindowManagerError;
use crate::operations::window_manager::ApplyConfig;
use crate::profile::{self, Profile};
use crate::setup::shortcuts;
use crate::setup::tray;
use crate::window_manager;
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

#[tauri::command]
pub fn profile_get<R: Runtime>(app_handle: AppHandle<R>) -> Result<Vec<Profile>, ProfileError> {
    profile::load_profiles(&app_handle)
}

#[tauri::command]
pub fn profile_apply(profile: Profile, pid: Option<u32>) -> Result<(), WindowManagerError> {
    let config = ApplyConfig::new().pid(pid).retry(true).monitor(true);
    window_manager::apply_profile(&profile, config)
}

#[tauri::command]
pub fn profile_test(profile: Profile, pid: Option<u32>) -> Result<(), WindowManagerError> {
    let config = ApplyConfig::new().pid(pid).retry(false).monitor(false);
    window_manager::apply_profile(&profile, config)
}

#[tauri::command]
pub fn profile_add<R: Runtime>(
    profile: Profile,
    app_handle: AppHandle<R>,
) -> Result<(), ProfileError> {
    let result = profile::add_profile(profile, &app_handle);
    tray::rebuild_tray_menu(&app_handle);
    shortcuts::rebuild_shortcuts(&app_handle);
    result
}

#[tauri::command]
pub fn profile_update<R: Runtime>(
    profile: Profile,
    app_handle: AppHandle<R>,
) -> Result<(), ProfileError> {
    let result = profile::update_profile(profile, &app_handle);
    tray::rebuild_tray_menu(&app_handle);
    shortcuts::rebuild_shortcuts(&app_handle);
    result
}

#[tauri::command]
pub fn profile_delete<R: Runtime>(
    profile: Profile,
    app_handle: AppHandle<R>,
) -> Result<(), ProfileError> {
    let result = profile::delete_profile(profile, &app_handle);
    tray::rebuild_tray_menu(&app_handle);
    shortcuts::rebuild_shortcuts(&app_handle);
    result
}

#[tauri::command]
pub fn profile_legacy_available<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<bool, ProfileError> {
    profile::has_importable_legacy_profiles(&app_handle)
}

#[tauri::command]
pub fn profile_import_legacy<R: Runtime>(
    app_handle: AppHandle<R>,
) -> Result<usize, ProfileError> {
    let result = profile::import_legacy_profiles(&app_handle);
    tray::rebuild_tray_menu(&app_handle);
    shortcuts::rebuild_shortcuts(&app_handle);
    result
}

#[tauri::command]
pub fn profile_reorder<R: Runtime>(
    uuids: Vec<Uuid>,
    app_handle: AppHandle<R>,
) -> Result<(), ProfileError> {
    let result = profile::reorder_profiles(uuids, &app_handle);
    tray::rebuild_tray_menu(&app_handle);
    shortcuts::rebuild_shortcuts(&app_handle);
    result
}
