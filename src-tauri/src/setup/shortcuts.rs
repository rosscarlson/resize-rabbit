use std::collections::HashMap;

use tauri::{AppHandle, GlobalShortcutManager, Manager, Runtime};

use crate::debug_log;
use crate::operations::process;
use crate::operations::profile::Profile;
use crate::operations::window_manager::{self, ApplyConfig};
use crate::setup::state::AppState;

/// Applies whichever of `candidates` actually has a game running, so the same
/// shortcut can be bound to more than one profile (e.g. one per game) as long
/// as only one of them is ever running at a time. With just one candidate
/// (the common case) this behaves exactly as before — no running-process
/// pre-check, straight to `apply_profile` including its own retry/window-
/// title-fallback logic — so single-profile shortcuts can't regress.
fn handle_shortcut_press(shortcut: &str, candidates: &[Profile]) {
    if candidates.len() == 1 {
        let _ = window_manager::apply_profile(
            &candidates[0],
            ApplyConfig::new().retry(true).monitor(true),
        );
        return;
    }

    match candidates
        .iter()
        .find(|profile| !process::get_pids_from_profile(profile).is_empty())
    {
        Some(profile) => {
            debug_log!(
                "Shortcut '{}' pressed — {} profiles share it, applying '{}' (its process is running)",
                shortcut,
                candidates.len(),
                profile.name
            );
            let _ = window_manager::apply_profile(
                profile,
                ApplyConfig::new().retry(true).monitor(true),
            );
        }
        None => {
            debug_log!(
                "Shortcut '{}' pressed but none of the {} profiles sharing it have a running process — ignoring this press.",
                shortcut,
                candidates.len()
            );
        }
    }
}

pub fn rebuild_shortcuts<R: Runtime>(app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();
    let profiles = state.profiles.lock().unwrap().clone();
    drop(state);

    let mut mgr = app_handle.global_shortcut_manager();
    let _ = mgr.unregister_all();

    // Group by shortcut string first — registering the same accelerator twice
    // would silently overwrite the first registration's callback (tauri's
    // global shortcut manager keys its internal listener map by the
    // accelerator itself), so exactly one native hotkey gets registered per
    // unique shortcut, covering every profile that uses it.
    let mut profiles_by_shortcut: HashMap<String, Vec<Profile>> = HashMap::new();

    for profile in profiles {
        let shortcut = match &profile.shortcut {
            Some(s) if !s.is_empty() => s.clone(),
            _ => continue,
        };

        profiles_by_shortcut.entry(shortcut).or_default().push(profile);
    }

    for (shortcut, candidates) in profiles_by_shortcut {
        let shortcut_clone = shortcut.clone();
        let result = mgr.register(&shortcut, move || {
            handle_shortcut_press(&shortcut_clone, &candidates);
        });

        if let Err(e) = result {
            eprintln!("Failed to register shortcut '{}': {}", shortcut, e);
        }
    }
}
