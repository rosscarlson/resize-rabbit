use tauri::{
    AppHandle, Builder, CustomMenuItem, Manager, Runtime, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu,
};
use uuid::Uuid;

use crate::operations::profile::Profile;
use crate::operations::window_manager::{self, ApplyConfig};
use crate::setup::state::AppState;

pub fn build_tray_menu(profiles: &[Profile]) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Show"))
        .add_native_item(SystemTrayMenuItem::Separator);

    for profile in profiles {
        let mut submenu_menu = SystemTrayMenu::new()
            .add_item(CustomMenuItem::new(format!("apply-{}", profile.uuid), "Apply"));

        if let Some(s) = &profile.shortcut {
            if !s.is_empty() {
                submenu_menu = submenu_menu.add_item(
                    CustomMenuItem::new(format!("shortcut-{}", profile.uuid), s).disabled(),
                );
            }
        }

        menu = menu.add_submenu(SystemTraySubmenu::new(&profile.name, submenu_menu));
    }

    menu.add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("check-updates", "Check for Updates"))
        .add_item(CustomMenuItem::new("exit", "Exit"))
}

pub fn rebuild_tray_menu<R: Runtime>(app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();
    let profiles = state.profiles.lock().unwrap().clone();
    let _ = app_handle.tray_handle().set_menu(build_tray_menu(&profiles));
}

pub fn setup_tray<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let system_tray = SystemTray::new().with_menu(build_tray_menu(&[]));

    builder
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                "exit" => {
                    app.exit(0);
                }
                "check-updates" => {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let window = app_clone.get_window("main");
                        match app_clone.updater().check().await {
                            Ok(update) if update.is_update_available() => {
                                let msg = format!(
                                    "Version {} is available. Install now?",
                                    update.latest_version()
                                );
                                let should_install = tauri::api::dialog::blocking::ask(
                                    window.as_ref(),
                                    "Update Available",
                                    msg,
                                );
                                if should_install {
                                    match update.download_and_install().await {
                                        Ok(_) => app_clone.restart(),
                                        Err(e) => tauri::api::dialog::blocking::message(
                                            window.as_ref(),
                                            "Update Error",
                                            format!("Failed to install update: {}", e),
                                        ),
                                    }
                                }
                            }
                            Ok(_) => {
                                tauri::api::dialog::blocking::message(
                                    window.as_ref(),
                                    "Up to Date",
                                    "You are running the latest version.",
                                );
                            }
                            Err(e) => {
                                tauri::api::dialog::blocking::message(
                                    window.as_ref(),
                                    "Update Check Failed",
                                    format!("Could not check for updates: {}", e),
                                );
                            }
                        }
                    });
                }
                other if other.starts_with("apply-") => {
                    let uuid_str = other.strip_prefix("apply-").unwrap();
                    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                        let state = app.state::<AppState>();
                        let profiles = state.profiles.lock().unwrap();
                        if let Some(profile) = profiles.iter().find(|p| p.uuid == uuid) {
                            let _ = window_manager::apply_profile(
                                profile,
                                ApplyConfig::new().retry(true).monitor(true),
                            );
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        })
}
