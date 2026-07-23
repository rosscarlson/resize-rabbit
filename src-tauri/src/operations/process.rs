extern crate base64;
extern crate image;
extern crate winapi;

use crate::debug_log;
use crate::debug_log_level;
use crate::debug_utils::DebugLevel;
use crate::operations::window_manager::ApplyConfig;
use crate::profile::Profile;
use crate::setup::tray;
use crate::window_manager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use sysinfo::ProcessRefreshKind;
use sysinfo::RefreshKind;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use tauri::{AppHandle, Runtime};
use winapi::shared::minwindef::{DWORD, LPARAM};
use winapi::um::handleapi::CloseHandle;
use winapi::um::minwinbase::STILL_ACTIVE;
use winapi::um::processthreadsapi::GetExitCodeProcess;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessInfo {
    name: String,
    pid: usize,
}

extern "system" fn enum_windows_callback(
    hwnd: winapi::shared::windef::HWND,
    lparam: LPARAM,
) -> i32 {
    let mut process_id: DWORD = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
        // Check if the window is visible before adding it to the set.
        if IsWindowVisible(hwnd) != 0 {
            let process_ids = &mut *(lparam as *mut HashSet<usize>);
            process_ids.insert(process_id as usize);
        }
    }
    1 // Continue enumeration
}

pub fn get_process_list(show_all: bool) -> Vec<ProcessInfo> {
    let system =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

    if show_all {
        return system
            .processes()
            .values()
            .map(|process| ProcessInfo {
                name: process.name().to_string(),
                pid: process.pid().as_u32() as usize,
            })
            .collect();
    }

    let mut process_ids_with_windows: HashSet<usize> = HashSet::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            &mut process_ids_with_windows as *mut _ as LPARAM,
        );
    }

    system
        .processes()
        .values()
        .filter(|p| process_ids_with_windows.contains(&(p.pid().as_u32() as usize)))
        .map(|process| ProcessInfo {
            name: process.name().to_string(),
            pid: process.pid().as_u32() as usize,
        })
        .collect()
}

/// Returns *every* running PID matching the profile's process name, not just one.
///
/// Some games run more than one process sharing the exact same executable name (a
/// launcher/parent alongside the real game process, or an anti-cheat relaunch) —
/// only one of which actually owns a visible top-level window. Picking a single
/// match (the old behavior) could silently pick the wrong one and then fail to
/// find any window at all, even though the process name genuinely matched and the
/// game was genuinely running with a visible, draggable window under a sibling
/// PID. Matching against *all* processes here (not just ones with a
/// currently-enumerable window) is also deliberate: the actual "does it have a
/// visible window" check happens later, once a PID is known, in
/// window_manager's own EnumWindows pass — this just needs to find candidates.
pub fn get_pids_from_profile(profile: &Profile) -> Vec<u32> {
    let target = profile.process_name.to_lowercase();
    let matched: Vec<u32> = get_process_list(true)
        .into_iter()
        .filter(|p| p.name.to_lowercase() == target)
        .map(|p| p.pid as u32)
        .collect();

    if matched.is_empty() {
        debug_log!(
            "Could not resolve process '{}' to any running PID",
            profile.process_name
        );
    } else {
        debug_log!(
            "Resolved process '{}' to PID(s): {:?}",
            profile.process_name,
            matched
        );
    }

    matched
}

pub fn watcher<R: Runtime>(
    watcher_flag: Arc<AtomicBool>,
    poll_rate_flag: Arc<AtomicU64>,
    profiles: Arc<Mutex<Vec<Profile>>>,
    app_handle: AppHandle<R>,
) {
    debug_log!(
        "Initial watcher_flag state: {}",
        watcher_flag.load(Ordering::SeqCst)
    );

    let mut applied_profiles: HashSet<u32> = HashSet::new();
    let mut running_profile_names: HashSet<String> = HashSet::new();

    let process_refresh_kind = ProcessRefreshKind::new();
    let refresh_kind = RefreshKind::new().with_processes(process_refresh_kind);

    let mut system = System::new_with_specifics(refresh_kind);

    loop {
        // Always refresh the process list, regardless of the watcher_flag
        // below (which only gates auto-apply-on-launch) — keeping the tray
        // menu's "currently running" pins fresh is a separate, always-on
        // convenience and shouldn't depend on that opt-in feature.
        system.refresh_processes_specifics(process_refresh_kind);
        let process_list: Vec<_> = system.processes().values().collect();

        let profiles_guard = profiles.lock().unwrap();
        let all_profiles: Vec<Profile> = profiles_guard.clone();
        drop(profiles_guard);

        let now_running: HashSet<String> = all_profiles
            .iter()
            .map(|p| p.process_name.to_lowercase())
            .filter(|name| {
                process_list
                    .iter()
                    .any(|proc| proc.name().to_lowercase() == *name)
            })
            .collect();

        if now_running != running_profile_names {
            running_profile_names = now_running;
            tray::rebuild_tray_menu(&app_handle);
        }

        // Check the flag to see if we should perform the watching logic
        if watcher_flag.load(Ordering::SeqCst) {
            // Only use profiles that have auto enabled
            let current_profiles: Vec<Profile> =
                all_profiles.iter().filter(|p| p.auto).cloned().collect();

            debug_log_level!(
                DebugLevel::Verbose,
                "Current profiles: {:?}",
                current_profiles
            );

            for profile in current_profiles.iter() {
                // find a matching process
                let process = process_list
                    .iter()
                    .find(|p| p.name().to_lowercase() == profile.process_name.to_lowercase());

                if process.is_none() {
                    debug_log_level!(
                        DebugLevel::Verbose,
                        "Could not find process: {}",
                        profile.process_name
                    );
                    continue;
                }

                debug_log_level!(DebugLevel::Verbose, "Found process: {:?}", process);

                let process_pid = process.unwrap().pid().as_u32();

                if applied_profiles.contains(&process_pid) {
                    continue;
                }

                debug_log!(
                    "Scheduling profile application for {} in {} ms",
                    process.unwrap().name(),
                    profile.delay
                );

                applied_profiles.insert(process_pid);

                let profile_clone = profile.clone();

                thread::Builder::new()
                    .name("Process Watcher - Apply".to_string())
                    .spawn(move || {
                        std::thread::sleep(Duration::from_millis(profile_clone.delay as u64));

                        // Deliberately not pinning `.pid(Some(process_pid))` here: that
                        // PID came from a naive process-name match (this thread's own
                        // `find()` above), same as what apply_profile's process-name path
                        // does internally — but passing it explicitly skips
                        // apply_profile's window-title fallback entirely (it only runs
                        // when no PID is given), which is what actually makes Test/Apply
                        // from the UI work for games like Forza Horizon 4 where the real
                        // window is owned by a different PID than the named process.
                        // Leaving `.pid()` unset lets apply_profile re-resolve fully,
                        // fallback included, matching the UI's working behavior exactly.
                        let _ = window_manager::apply_profile(
                            &profile_clone,
                            ApplyConfig::new().retry(true).monitor(true),
                        );
                    })
                    .unwrap();
            }

            // Remove any potential applied profiles that are no longer running
            let mut to_remove: Vec<u32> = Vec::new();

            for applied_profile in applied_profiles.iter() {
                let process = process_list
                    .iter()
                    .find(|p| p.pid().as_u32() == *applied_profile);

                if process.is_none() {
                    to_remove.push(*applied_profile);
                }
            }

            for pid in to_remove.iter() {
                applied_profiles.remove(pid);
            }

            debug_log_level!(
                DebugLevel::Verbose,
                "Watching {}",
                chrono::Local::now().format("%H:%M:%S")
            );
        } else {
            debug_log_level!(
                DebugLevel::Verbose,
                "Watcher is on standby. {}",
                chrono::Local::now().format("%H:%M:%S")
            );
        }

        // Sleep for a short duration before checking the flag again
        thread::sleep(Duration::from_millis(poll_rate_flag.load(Ordering::SeqCst)));
    }
}

pub fn is_process_running(pid: DWORD) -> bool {
    let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };

    if !process_handle.is_null() {
        let mut exit_code: DWORD = 0;
        let result = unsafe { GetExitCodeProcess(process_handle, &mut exit_code) };
        unsafe { CloseHandle(process_handle) };

        result != 0 && exit_code == STILL_ACTIVE
    } else {
        false
    }
}
