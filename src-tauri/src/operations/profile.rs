extern crate uuid;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::api::path;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

use crate::errors::profile::Error as ProfileError;
use crate::setup::state::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Profile {
    pub uuid: Uuid,
    pub name: String,
    pub process_name: String,
    pub auto: bool,
    pub delay: i32,
    pub window_height: i32,
    pub window_width: i32,
    pub window_pos_y: i32,
    pub window_pos_x: i32,
    pub remove_borders: bool,
    /// Only meaningful when `remove_borders` is set. Pushes the window up and
    /// grows its height by however much leftover title-bar chrome we can
    /// measure, so the chrome lands off the top edge of the display instead of
    /// overlapping the game. See `window_manager::effective_target_rect` — it's
    /// strictly gated to windows we can actually measure (confirmed UWP
    /// `ApplicationFrameWindow` targets), so this is a no-op for every other
    /// game even if left on.
    pub shift_titlebar_offscreen: bool,
    pub shortcut: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "New Profile".to_string(),
            process_name: "".to_string(),
            auto: false,
            delay: 0,
            window_height: 600,
            window_width: 800,
            window_pos_y: 0,
            window_pos_x: 0,
            remove_borders: false,
            shift_titlebar_offscreen: false,
            shortcut: None,
        }
    }
}

pub fn get_profiles_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf, ProfileError> {
    let settings_path =
        path::app_data_dir(&app_handle.config()).ok_or(ProfileError::ProfilePathError)?;

    if !settings_path.exists() {
        fs::create_dir_all(&settings_path)?;
    }

    Ok(settings_path.join("profiles.json"))
}

pub fn load_profiles<R: Runtime>(app_handle: &AppHandle<R>) -> Result<Vec<Profile>, ProfileError> {
    let profile_path = get_profiles_path(app_handle)?;

    // If profiles doesnt exist we should create them with an empty object
    if !profile_path.exists() {
        let profiles: Vec<Profile> = vec![];
        let json_string: String = serde_json::to_string_pretty(&profiles)?;
        fs::write(&profile_path, json_string)?;
    }

    let profiles_json = fs::read_to_string(profile_path)?;
    serde_json::from_str(&profiles_json).map_err(Into::into)
}

fn save_profiles_to_disk<R: Runtime>(
    profiles: &Vec<Profile>,
    app_handle: &AppHandle<R>,
) -> Result<(), ProfileError> {
    let json_string: String = serde_json::to_string_pretty(&profiles)?;
    let profiles_path = get_profiles_path(app_handle)?;

    fs::write(profiles_path, json_string).map_err(Into::into)
}

fn update_profiles_state<R: Runtime>(profiles: Vec<Profile>, app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();

    // Lock the state, replace the profiles list with the new list
    {
        let mut app_state = state.profiles.lock().unwrap();
        *app_state = profiles;
    } // Lock is automatically released here
}

pub fn add_profile<R: Runtime>(profile: Profile, app: &AppHandle<R>) -> Result<(), ProfileError> {
    let mut profiles = load_profiles(app)?;
    profiles.push(profile);

    save_profiles_to_disk(&profiles, app)?;
    update_profiles_state(profiles, app);

    Ok(())
}

pub fn update_profile<R: Runtime>(
    profile: Profile,
    app: &AppHandle<R>,
) -> Result<(), ProfileError> {
    let mut profiles = load_profiles(app)?;
    let index = profiles
        .iter()
        .position(|p| p.uuid == profile.uuid)
        .ok_or(ProfileError::NotFound)?;
    profiles[index] = profile;

    save_profiles_to_disk(&profiles, app)?;
    update_profiles_state(profiles, app);

    Ok(())
}

pub fn delete_profile<R: Runtime>(
    profile: Profile,
    app: &AppHandle<R>,
) -> Result<(), ProfileError> {
    let mut profiles = load_profiles(app)?;
    let index = profiles
        .iter()
        .position(|p| p.uuid == profile.uuid)
        .ok_or(ProfileError::NotFound)?;
    profiles.remove(index);

    save_profiles_to_disk(&profiles, app)?;
    update_profiles_state(profiles, app);

    Ok(())
}

pub fn has_importable_legacy_profiles<R: Runtime>(app: &AppHandle<R>) -> Result<bool, ProfileError> {
    let legacy_path = std::env::var("APPDATA")
        .map(|p| std::path::PathBuf::from(p).join("com.resizeraccoon.dev").join("profiles.json"))
        .map_err(|_| ProfileError::ProfilePathError)?;

    if !legacy_path.exists() {
        return Ok(false);
    }

    let legacy_json = fs::read_to_string(legacy_path)?;
    let legacy_profiles: Vec<Profile> = serde_json::from_str(&legacy_json)?;

    let current = load_profiles(app)?;
    let existing_uuids: std::collections::HashSet<Uuid> =
        current.iter().map(|p| p.uuid).collect();

    Ok(legacy_profiles
        .iter()
        .any(|p| !existing_uuids.contains(&p.uuid)))
}

pub fn import_legacy_profiles<R: Runtime>(app: &AppHandle<R>) -> Result<usize, ProfileError> {
    let legacy_path = std::env::var("APPDATA")
        .map(|p| std::path::PathBuf::from(p).join("com.resizeraccoon.dev").join("profiles.json"))
        .map_err(|_| ProfileError::ProfilePathError)?;

    if !legacy_path.exists() {
        return Ok(0);
    }

    let legacy_json = fs::read_to_string(legacy_path)?;
    let legacy_profiles: Vec<Profile> = serde_json::from_str(&legacy_json)?;

    let mut current = load_profiles(app)?;
    let existing_uuids: std::collections::HashSet<Uuid> =
        current.iter().map(|p| p.uuid).collect();

    let new_profiles: Vec<Profile> = legacy_profiles
        .into_iter()
        .filter(|p| !existing_uuids.contains(&p.uuid))
        .collect();

    let count = new_profiles.len();
    current.extend(new_profiles);

    save_profiles_to_disk(&current, app)?;
    update_profiles_state(current, app);

    Ok(count)
}

pub fn reorder_profiles<R: Runtime>(
    uuids: Vec<Uuid>,
    app: &AppHandle<R>,
) -> Result<(), ProfileError> {
    let profiles = load_profiles(app)?;
    let reordered: Vec<Profile> = uuids
        .iter()
        .filter_map(|id| profiles.iter().find(|p| &p.uuid == id).cloned())
        .collect();

    save_profiles_to_disk(&reordered, app)?;
    update_profiles_state(reordered, app);

    Ok(())
}
