use tauri::{
    AppHandle, Builder, CustomMenuItem, Manager, Runtime, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu,
};
use uuid::Uuid;

use crate::operations::profile::Profile;
use crate::operations::user_settings;
use crate::operations::window_manager::{self, ApplyConfig};
use crate::setup::state::AppState;

const BUNDLED_LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../../locales/en.json")),
    ("es", include_str!("../../locales/es.json")),
    ("fr", include_str!("../../locales/fr.json")),
    ("de", include_str!("../../locales/de.json")),
    ("it", include_str!("../../locales/it.json")),
    ("pt", include_str!("../../locales/pt.json")),
    ("nl", include_str!("../../locales/nl.json")),
    ("pl", include_str!("../../locales/pl.json")),
    ("ru", include_str!("../../locales/ru.json")),
    ("zh", include_str!("../../locales/zh.json")),
    ("ja", include_str!("../../locales/ja.json")),
    ("ko", include_str!("../../locales/ko.json")),
];

pub struct TrayStrings {
    pub show: String,
    pub apply: String,
    pub check_updates: String,
    pub exit: String,
    pub update_available_title: String,
    pub update_available_msg: String,
    pub update_error_title: String,
    pub update_error_msg: String,
    pub up_to_date_title: String,
    pub up_to_date_msg: String,
    pub update_check_failed_title: String,
    pub update_check_failed_msg: String,
}

impl Default for TrayStrings {
    fn default() -> Self {
        TrayStrings {
            show: "Show".to_string(),
            apply: "Apply".to_string(),
            check_updates: "Check for Updates".to_string(),
            exit: "Exit".to_string(),
            update_available_title: "Update Available".to_string(),
            update_available_msg: "Version {{version}} is available. Install now?".to_string(),
            update_error_title: "Update Error".to_string(),
            update_error_msg: "Failed to install update: {{error}}".to_string(),
            up_to_date_title: "Up to Date".to_string(),
            up_to_date_msg: "You are running the latest version.".to_string(),
            update_check_failed_title: "Update Check Failed".to_string(),
            update_check_failed_msg: "Could not check for updates: {{error}}".to_string(),
        }
    }
}

fn str_or_default<'a>(val: &'a serde_json::Value, key: &str, default: &'a str) -> String {
    val["tray"][key].as_str().unwrap_or(default).to_string()
}

pub fn tray_strings_for_lang<R: Runtime>(lang: &str, app: &AppHandle<R>) -> TrayStrings {
    // AppData locale may be an older version without a "tray" section; only use it if that section exists
    let appdata_val = tauri::api::path::app_data_dir(&app.config())
        .map(|d| d.join("locales").join(format!("{}.json", lang)))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .filter(|v| !v["tray"].is_null());

    let val = appdata_val
        .or_else(|| {
            BUNDLED_LOCALES.iter()
                .find(|(l, _)| *l == lang)
                .and_then(|(_, c)| serde_json::from_str::<serde_json::Value>(c).ok())
        })
        .or_else(|| {
            BUNDLED_LOCALES.iter()
                .find(|(l, _)| *l == "en")
                .and_then(|(_, c)| serde_json::from_str::<serde_json::Value>(c).ok())
        });

    let Some(val) = val else {
        return TrayStrings::default();
    };

    TrayStrings {
        show: str_or_default(&val, "show", "Show"),
        apply: str_or_default(&val, "apply", "Apply"),
        check_updates: str_or_default(&val, "checkForUpdates", "Check for Updates"),
        exit: str_or_default(&val, "exit", "Exit"),
        update_available_title: str_or_default(&val, "updateAvailableTitle", "Update Available"),
        update_available_msg: str_or_default(&val, "updateAvailableMsg", "Version {{version}} is available. Install now?"),
        update_error_title: str_or_default(&val, "updateErrorTitle", "Update Error"),
        update_error_msg: str_or_default(&val, "updateErrorMsg", "Failed to install update: {{error}}"),
        up_to_date_title: str_or_default(&val, "upToDateTitle", "Up to Date"),
        up_to_date_msg: str_or_default(&val, "upToDateMsg", "You are running the latest version."),
        update_check_failed_title: str_or_default(&val, "updateCheckFailedTitle", "Update Check Failed"),
        update_check_failed_msg: str_or_default(&val, "updateCheckFailedMsg", "Could not check for updates: {{error}}"),
    }
}

fn current_lang<R: Runtime>(app: &AppHandle<R>) -> String {
    user_settings::get_user_settings(app)
        .map(|s| s.language)
        .unwrap_or_else(|_| "en".to_string())
}

pub fn build_tray_menu(profiles: &[Profile], strings: &TrayStrings) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", &*strings.show))
        .add_item(CustomMenuItem::new("check-updates", &*strings.check_updates))
        .add_item(CustomMenuItem::new("exit", &*strings.exit))
        .add_native_item(SystemTrayMenuItem::Separator);

    for profile in profiles {
        let mut submenu_menu = SystemTrayMenu::new()
            .add_item(CustomMenuItem::new(format!("apply-{}", profile.uuid), &*strings.apply));

        if let Some(s) = &profile.shortcut {
            if !s.is_empty() {
                submenu_menu = submenu_menu.add_item(
                    CustomMenuItem::new(format!("shortcut-{}", profile.uuid), s).disabled(),
                );
            }
        }

        menu = menu.add_submenu(SystemTraySubmenu::new(&profile.name, submenu_menu));
    }

    menu
}

pub fn rebuild_tray_menu<R: Runtime>(app_handle: &AppHandle<R>) {
    let state = app_handle.state::<AppState>();
    let profiles = state.profiles.lock().unwrap().clone();
    drop(state);
    let lang = current_lang(app_handle);
    let strings = tray_strings_for_lang(&lang, app_handle);
    let _ = app_handle.tray_handle().set_menu(build_tray_menu(&profiles, &strings));
}

pub fn setup_tray<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let system_tray = SystemTray::new().with_menu(build_tray_menu(&[], &TrayStrings::default()));

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
                    let lang = current_lang(app);
                    let strings = tray_strings_for_lang(&lang, app);
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let window = app_clone.get_window("main");
                        match app_clone.updater().check().await {
                            Ok(update) if update.is_update_available() => {
                                let msg = strings.update_available_msg
                                    .replace("{{version}}", update.latest_version());
                                let should_install = tauri::api::dialog::blocking::ask(
                                    window.as_ref(),
                                    &strings.update_available_title,
                                    &msg,
                                );
                                if should_install {
                                    match update.download_and_install().await {
                                        Ok(_) => app_clone.restart(),
                                        Err(e) => tauri::api::dialog::blocking::message(
                                            window.as_ref(),
                                            &strings.update_error_title,
                                            &strings.update_error_msg.replace("{{error}}", &e.to_string()),
                                        ),
                                    }
                                }
                            }
                            Ok(_) => {
                                tauri::api::dialog::blocking::message(
                                    window.as_ref(),
                                    &strings.up_to_date_title,
                                    &strings.up_to_date_msg,
                                );
                            }
                            Err(e) => {
                                tauri::api::dialog::blocking::message(
                                    window.as_ref(),
                                    &strings.update_check_failed_title,
                                    &strings.update_check_failed_msg.replace("{{error}}", &e.to_string()),
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
