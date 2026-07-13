use tauri::{generate_handler, Builder, Runtime};

pub mod locale_commands;
pub mod process_commands;
pub mod profile_commands;
pub mod settings_commands;

pub fn register_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(generate_handler![
        profile_commands::profile_get,
        profile_commands::profile_apply,
        profile_commands::profile_test,
        profile_commands::profile_add,
        profile_commands::profile_update,
        profile_commands::profile_delete,
        profile_commands::profile_reorder,
        profile_commands::profile_import_legacy,
        profile_commands::profile_legacy_available,
        process_commands::process_get,
        settings_commands::settings_get,
        settings_commands::settings_update,
        settings_commands::settings_toggle_launch_on_start,
        locale_commands::locale_list,
        locale_commands::locale_load,
    ])
}
