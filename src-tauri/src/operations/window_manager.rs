use std::cell::RefCell;
use std::ptr::{self, NonNull};
use std::thread;
use std::time::Duration;

use crate::debug_log;
use crate::errors::window_manager::Error as WindowManagerError;
use crate::process::is_process_running;
use crate::profile::Profile;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPARAM, TRUE};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    EnumChildWindows, EnumWindows, GetClassNameW, GetWindowLongPtrW, GetWindowLongW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, MoveWindow, SetWindowLongPtrW, SetWindowPos,
    ShowWindow, GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, WS_BORDER,
    WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE, WS_EX_WINDOWEDGE,
    WS_THICKFRAME, WS_VISIBLE,
};

thread_local! {
    static HOOK_DATA: RefCell<Option<Profile>> = RefCell::new(None);
}

fn is_target_window(hwnd: HWND, target_pid: DWORD) -> bool {
    let mut window_pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut window_pid);
    }
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 };

    window_pid == target_pid && style & WS_VISIBLE != 0
}

fn debug_window(hwnd: HWND) {
    let mut class_name = [0u16; 256]; // Adjust size as necessary
    unsafe {
        GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    }
    let class_name = String::from_utf16_lossy(&class_name);
    debug_log!("Class name: {}", class_name);

    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }

    debug_log!(
        "RECT - Left: {}, Top: {}, Right: {}, Bottom: {}",
        rect.left,
        rect.top,
        rect.right,
        rect.bottom
    );
}

fn validate_window_rect(hwnd: HWND, profile: &Profile) -> bool {
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        GetWindowRect(hwnd, &mut rect);
    }

    let actual_x = rect.left;
    let actual_y = rect.top;
    let actual_width = rect.right - rect.left;
    let actual_height = rect.bottom - rect.top;

    let (intended_x, intended_y, intended_width, intended_height) =
        effective_target_rect(hwnd, profile);

    actual_x == intended_x
        && actual_y == intended_y
        && actual_width == intended_width
        && actual_height == intended_height
}

extern "system" fn measure_core_window_top_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if window_class_name(hwnd) != "Windows.UI.Core.CoreWindow" {
        return TRUE;
    }

    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe { GetWindowRect(hwnd, &mut rect) };

    let out_ptr = lparam as *mut i32;
    unsafe { *out_ptr = rect.top };

    FALSE // found it, stop enumerating
}

/// Measures how far the game's real content (the "Windows.UI.Core.CoreWindow"
/// child, confirmed via a child-window dump for Forza Horizon 4 — see the
/// maintainer README's UWP title-bar investigation) sits below the frame's
/// own top edge. Pure read (`GetWindowRect` only) — never `MoveWindow`/
/// `SetWindowPos` on the CoreWindow itself, since resizing that child
/// directly was tried and reverted earlier after it broke the game's
/// swapchain on a multi-GPU setup.
///
/// Deliberately measures the CoreWindow's own offset from the frame's top,
/// not the "ApplicationFrameTitleBarWindow" strip's height (the first cut of
/// this code measured the strip and consistently left a 1px sliver visible):
/// the dump showed the strip itself starting 1px below the frame's top edge
/// (rect (_, 1, _, 33) inside a frame whose own top was 0) — some sliver of
/// frame edge/border sits above the strip that isn't part of it. Measuring
/// from the actual content's position rather than the strip's bounds sidesteps
/// that gap entirely instead of needing to special-case it.
///
/// Returns `None` — never a guessed fallback value — for anything that isn't
/// a genuine "ApplicationFrameWindow" frame with a CoreWindow child actually
/// present, so this can only ever affect windows we've confirmed have this
/// specific shell chrome, not classic games whose border removal already
/// works cleanly.
fn measure_uwp_content_top_offset(hwnd: HWND) -> Option<i32> {
    if window_class_name(hwnd) != "ApplicationFrameWindow" {
        return None;
    }

    let mut frame_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe { GetWindowRect(hwnd, &mut frame_rect) };

    let mut core_window_top: i32 = i32::MIN;
    unsafe {
        EnumChildWindows(
            hwnd,
            Some(measure_core_window_top_callback),
            &mut core_window_top as *mut i32 as LPARAM,
        );
    }

    if core_window_top == i32::MIN {
        return None; // no CoreWindow child found
    }

    let offset = core_window_top - frame_rect.top;

    if offset > 0 {
        Some(offset)
    } else {
        None
    }
}

/// The rect we actually move/validate the window to. Normally just the
/// profile's raw values. When `shift_titlebar_offscreen` is set *and* we can
/// measure a real leftover UWP title-bar strip on this exact window, the rect
/// is shifted up and grown taller by that measured amount instead — the
/// visible game content (which sits at a fixed offset inside the frame, below
/// the strip) ends up landing exactly on the profile's intended rect, while
/// the strip itself lands above the display's top edge. Falls back to the raw
/// profile rect whenever nothing UWP-specific was measured, so this is a
/// guaranteed no-op for every other game regardless of the checkbox.
fn effective_target_rect(hwnd: HWND, profile: &Profile) -> (i32, i32, i32, i32) {
    let base = (
        profile.window_pos_x,
        profile.window_pos_y,
        profile.window_width,
        profile.window_height,
    );

    if !profile.shift_titlebar_offscreen {
        return base;
    }

    match measure_uwp_content_top_offset(hwnd) {
        Some(shift) => (
            profile.window_pos_x,
            profile.window_pos_y - shift,
            profile.window_width,
            profile.window_height + shift,
        ),
        None => base,
    }
}

fn remove_window_borders(hwnd: HWND, profile: &Profile) {
    if profile.remove_borders == false {
        return;
    }

    // Remove standard window styles
    let mut style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    style |= (WS_THICKFRAME | WS_DLGFRAME | WS_BORDER) as isize;
    style ^= (WS_THICKFRAME | WS_DLGFRAME | WS_BORDER) as isize;
    unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, style) };

    // Remove extended window styles
    let mut ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    ex_style |=
        (WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE) as isize;
    ex_style ^=
        (WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE) as isize;
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style) };

    // Adjust window position to refresh the appearance
    let u_flags = SWP_NOSIZE
        | SWP_NOMOVE
        | SWP_NOZORDER
        | SWP_NOACTIVATE
        | SWP_NOOWNERZORDER
        | SWP_NOSENDCHANGING
        | SWP_FRAMECHANGED;
    unsafe { SetWindowPos(hwnd, ptr::null_mut(), 0, 0, 0, 0, u_flags) };
}

fn window_class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len as usize])
}

extern "system" fn hide_titlebar_child_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let pid = lparam as DWORD;
    let class = window_class_name(hwnd);

    if class != "ApplicationFrameTitleBarWindow" {
        return TRUE;
    }

    let hidden = unsafe { ShowWindow(hwnd, SW_HIDE) };
    debug_log!(
        "PID {}: hid ApplicationFrameTitleBarWindow child (was previously visible: {})",
        pid,
        hidden != 0
    );

    TRUE
}

/// UWP-hosted windows (class "ApplicationFrameWindow" — confirmed for Forza
/// Horizon 4) draw their title bar / back-button strip using a genuinely
/// separate child window, class "ApplicationFrameTitleBarWindow" — confirmed via
/// a child-window dump Ross pasted back (rect (-2560, 1, 7680, 33), sitting right
/// above the game's own "Windows.UI.Core.CoreWindow" child at (-2560, 33, 7680,
/// 1440)). Not classic non-client caption (WS_CAPTION removal, in
/// `remove_window_borders` above, has no effect on it) and not DWM-drawn frame
/// chrome either (DWMWA_NCRENDERING_POLICY=DISABLED, tried first, returned
/// success but changed nothing visible) — it's an ordinary sibling HWND.
///
/// Fix: hide that specific child outright via `ShowWindow(..., SW_HIDE)`. This
/// never touches the CoreWindow (the game's actual rendering surface) at all —
/// no resize, no reposition, nothing the game itself has to renegotiate — so it
/// can't cause the performance regression the earlier CoreWindow-resize attempt
/// did. Gated on the frame's class being exactly "ApplicationFrameWindow", so
/// this never runs at all for classic Win32 games (Elite Dangerous, Forza 5/6,
/// etc) — confirmed nothing here can affect them.
fn hide_uwp_titlebar(hwnd: HWND, pid: DWORD) {
    if window_class_name(hwnd) != "ApplicationFrameWindow" {
        return;
    }

    unsafe {
        EnumChildWindows(
            hwnd,
            Some(hide_titlebar_child_callback),
            pid as LPARAM,
        );
    }
}

fn move_and_validate_window(
    hwnd: HWND,
    profile: &Profile,
    pid: DWORD,
) -> Result<(), WindowManagerError> {
    remove_window_borders(hwnd, profile);

    let (target_x, target_y, target_width, target_height) = effective_target_rect(hwnd, profile);
    if profile.shift_titlebar_offscreen && (target_y, target_height) != (profile.window_pos_y, profile.window_height) {
        debug_log!(
            "PID {}: shifting titlebar off-screen — measured {}px, target rect now ({}, {}, {}, {})",
            pid,
            target_height - profile.window_height,
            target_x,
            target_y,
            target_width,
            target_height
        );
    }

    let moved = unsafe {
        MoveWindow(
            hwnd,
            target_x,
            target_y,
            target_width,
            target_height,
            TRUE,
        )
    };

    if moved == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code == 5 {
            debug_log!(
                "Failed to move and resize window for PID {}: Access denied.",
                pid
            );
            return Err(WindowManagerError::AccessDenied);
        } else {
            debug_log!(
                "Failed to move and resize window for PID {}: Error code: {}",
                pid,
                error_code
            );
            return Err(WindowManagerError::ApplyFailed);
        }
    }

    if !validate_window_rect(hwnd, profile) {
        debug_log!(
            "Failed to move and resize window for PID {}: Window was not moved correctly.",
            pid
        );
        return Err(WindowManagerError::ApplyFailed);
    }

    if profile.remove_borders {
        hide_uwp_titlebar(hwnd, pid);
    }

    Ok(())
}

fn watch_for_profile_overrides(hwnd: HWND, profile: &Profile, pid: DWORD) {
    let profile_clone = profile.clone();
    let hwnd_as_int = hwnd as usize;

    thread::Builder::new()
        .name("Profile override watcher".to_string())
        .spawn(move || {
            let mut poll_counter = 0;
            let mut poll_extended_counter = 0;

            let poll_itterations = 10;
            let poll_extended_itterations = 5;

            let hwnd_clone = hwnd_as_int as HWND;

            loop {
                if !is_process_running(pid) {
                    debug_log!(
                        "[{}] Process no longer running, exiting override polling",
                        profile_clone.name
                    );
                    break;
                }

                // check pos
                let matches_profile = validate_window_rect(hwnd_clone, &profile_clone);
                poll_counter += 1;

                if poll_counter <= poll_itterations {
                    debug_log!(
                        "[{}] Polling for profile overrides: [{}/10 x 1second]",
                        profile_clone.name,
                        poll_counter
                    );
                } else {
                    poll_extended_counter += 1;
                    debug_log!(
                        "[{}] Polling for profile overrides: [{}/5 x 5seconds]",
                        profile_clone.name,
                        poll_extended_counter
                    );
                }

                if !matches_profile {
                    debug_log!("Window for PID {} was moved or resized.", pid);
                    debug_window(hwnd_clone);

                    let result = move_and_validate_window(hwnd_clone, &profile_clone, pid);
                    if let Err(error) = result {
                        debug_log!("Failed to re-apply profile: {:?}", error);
                    }
                }

                if poll_counter < poll_itterations {
                    thread::sleep(Duration::from_secs(1));
                } else {
                    if poll_extended_counter < poll_extended_itterations {
                        thread::sleep(Duration::from_secs(5));
                    } else {
                        // we are done polling
                        debug_log!("Done polling for profile overrides.");
                        break;
                    }
                }
            }
        })
        .unwrap();
}

extern "system" fn dump_visible_windows_callback(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 };
    if style & WS_VISIBLE == 0 {
        return TRUE;
    }

    let mut title_buf = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32) };
    if len <= 0 {
        return TRUE; // skip untitled windows, mostly background/helper noise
    }
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);

    let mut pid: DWORD = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };

    debug_log!("  window: PID {} — \"{}\"", pid, title);

    TRUE
}

/// Diagnostic dump of every visible, titled top-level window and its owning PID —
/// logged once we've exhausted every candidate PID for a profile with no luck, so
/// a pasted-back log shows what Windows itself thinks owns the game's window,
/// which may not match the PID a profile resolved to by process name.
fn log_visible_windows() {
    debug_log!("Currently visible top-level windows with a title:");
    unsafe {
        EnumWindows(Some(dump_visible_windows_callback), 0);
    }
}

extern "system" fn find_window_by_title_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let (needle_ptr, mut result_ptr): (*const String, NonNull<Option<DWORD>>) =
        unsafe { *(lparam as *const (*const String, NonNull<Option<DWORD>>)) };
    let needle = unsafe { &*needle_ptr };

    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 };
    if style & WS_VISIBLE == 0 {
        return TRUE;
    }

    let mut title_buf = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32) };
    if len <= 0 {
        return TRUE;
    }
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);

    if !title.to_lowercase().contains(&needle.to_lowercase()) {
        return TRUE;
    }

    let mut pid: DWORD = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    unsafe { *result_ptr.as_mut() = Some(pid) };

    FALSE // found it, stop enumerating
}

/// Some games (Forza Horizon 4 confirmed, via a pasted-back log) run their visible
/// window under a PID that belongs to a *different* executable name than the one
/// a profile is configured with — process-name matching can never find it. As a
/// last resort, look for any visible top-level window whose title contains the
/// profile's display name (which commonly matches the game's real window title,
/// e.g. a profile named "Forza Horizon 4" and a window titled "Forza Horizon 4")
/// and use whatever PID actually owns that window instead.
fn find_pid_by_window_title(name: &str) -> Option<DWORD> {
    if name.trim().is_empty() {
        return None;
    }

    let mut result: Option<DWORD> = None;
    let result_ptr = NonNull::new(&mut result).unwrap();
    let needle = name.to_string();
    let callback_data = (&needle as *const String, result_ptr);

    unsafe {
        EnumWindows(
            Some(find_window_by_title_callback),
            &callback_data as *const _ as LPARAM,
        );
    }

    result
}

extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let (pid, profile_ptr, mut success_ptr, mut error_ptr, monitor): (
        DWORD,
        *const Profile,
        NonNull<bool>,
        NonNull<Option<WindowManagerError>>,
        bool,
    ) = unsafe {
        *(lparam
            as *const (
                DWORD,
                *const Profile,
                NonNull<bool>,
                NonNull<Option<WindowManagerError>>,
                bool,
            ))
    };

    let profile = unsafe { &*profile_ptr };

    if is_target_window(hwnd, pid) {
        let result = move_and_validate_window(hwnd, profile, pid);
        debug_window(hwnd);
        match result {
            Ok(()) => {
                unsafe { *success_ptr.as_mut() = true };
                if monitor {
                    watch_for_profile_overrides(hwnd, profile, pid);
                }
            }
            Err(error) => unsafe {
                *success_ptr.as_mut() = false;
                *error_ptr.as_mut() = Some(error);
            },
        }
        return FALSE; // Stop enumeration because we've found and moved the window
    }

    TRUE // Continue enumeration if this window did not match
}

#[derive(Debug)]
pub struct ApplyConfig {
    pid: Option<u32>,
    retry: bool,
    retries: u8,
    monitor: bool,
}

impl Default for ApplyConfig {
    fn default() -> Self {
        ApplyConfig {
            pid: None,
            retry: false,
            retries: 0,
            monitor: false,
        }
    }
}

impl ApplyConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn pid(mut self, pid: Option<DWORD>) -> Self {
        self.pid = pid;
        self
    }

    pub fn retry(mut self, retry: bool) -> Self {
        self.retry = retry;
        self
    }

    pub fn retries(mut self, retries: u8) -> Self {
        self.retries = retries;
        self
    }

    pub fn monitor(mut self, monitor: bool) -> Self {
        self.monitor = monitor;
        self
    }
}

/// Runs one EnumWindows pass looking for a visible top-level window owned by `pid`.
/// `Ok(())` — found it and successfully moved/resized it.
/// `Err(Some(e))` — found a matching window but failed to apply the profile to it.
/// `Err(None)` — no matching window for this PID at all (caller should try the next candidate, if any).
fn apply_to_pid(
    profile: &Profile,
    pid: DWORD,
    monitor: bool,
) -> Result<(), Option<WindowManagerError>> {
    let mut success = false;
    let success_ptr = NonNull::new(&mut success).unwrap();

    let mut error = None;
    let error_ptr = NonNull::new(&mut error).unwrap();

    let callback_data = (pid, profile as *const _, success_ptr, error_ptr, monitor);

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            &callback_data as *const _ as LPARAM,
        );
    }

    if success {
        Ok(())
    } else {
        Err(error)
    }
}

pub fn apply_profile(profile: &Profile, config: ApplyConfig) -> Result<(), WindowManagerError> {
    // A process name can match more than one running PID (a launcher/parent process
    // alongside the real game process sharing the same exe name is common) — only
    // one of which may actually own a visible window. Try every candidate rather
    // than betting on whichever one sysinfo happened to resolve first.
    let candidate_pids: Vec<DWORD> = match config.pid {
        Some(pid) => vec![pid],
        None => crate::process::get_pids_from_profile(profile),
    };

    debug_log!("Profile config {:#?}", config);

    if candidate_pids.len() > 1 {
        debug_log!(
            "[{}] '{}' matched {} running processes ({:?}) — trying each until one has a window.",
            profile.name,
            profile.process_name,
            candidate_pids.len(),
            candidate_pids
        );
    }

    for pid in &candidate_pids {
        match apply_to_pid(profile, *pid, config.monitor) {
            Ok(()) => {
                debug_log!(
                    "Successfully applied profile: {} (PID {})",
                    profile.name,
                    pid
                );
                return Ok(());
            }
            Err(Some(error)) => {
                debug_log!(
                    "[{}] Found a window for PID {} but failed to apply: {:?}",
                    profile.name,
                    pid,
                    error
                );
                return Err(error);
            }
            Err(None) => {
                debug_log!(
                    "[{}] PID {} has no matching window, trying next candidate if any.",
                    profile.name,
                    pid
                );
            }
        }
    }

    // Some games run their visible window under a PID belonging to a different
    // executable than the one the profile is configured with (confirmed for Forza
    // Horizon 4) — process-name matching can never find it. Last resort: look for
    // any visible window whose title matches the profile's display name instead.
    if config.pid.is_none() {
        if let Some(pid) = find_pid_by_window_title(&profile.name) {
            debug_log!(
                "[{}] No window found via process name; found a window titled like '{}' owned by PID {} instead, trying that.",
                profile.name,
                profile.name,
                pid
            );
            match apply_to_pid(profile, pid, config.monitor) {
                Ok(()) => {
                    debug_log!(
                        "Successfully applied profile: {} (PID {}, matched by window title)",
                        profile.name,
                        pid
                    );
                    return Ok(());
                }
                Err(Some(error)) => {
                    debug_log!(
                        "[{}] Found a window by title (PID {}) but failed to apply: {:?}",
                        profile.name,
                        pid,
                        error
                    );
                    return Err(error);
                }
                Err(None) => {
                    debug_log!(
                        "[{}] Window-title fallback PID {} unexpectedly had no matching window on second pass.",
                        profile.name,
                        pid
                    );
                }
            }
        }
    }

    if candidate_pids.is_empty() {
        debug_log!(
            "[{}] No running process found matching '{}', and no window titled like '{}' either; cannot apply.",
            profile.name,
            profile.process_name,
            profile.name
        );
        return Err(WindowManagerError::ProcessNotFound);
    }

    if config.retry && config.retries < 2 {
        debug_log!(
            "[{}] Failed to find active window, retrying in 5 seconds...",
            profile.name
        );
        thread::sleep(Duration::from_secs(5));

        let retry_config = ApplyConfig::new()
            .pid(config.pid)
            .retry(true)
            .monitor(config.monitor)
            .retries(config.retries + 1);

        return apply_profile(profile, retry_config);
    }

    debug_log!(
        "[{}] Failed to find active window after {} attempt(s).",
        profile.name,
        config.retries + 1
    );
    log_visible_windows();
    Err(WindowManagerError::ApplyFailed)
}
