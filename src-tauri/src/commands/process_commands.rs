use crate::process::{self, ProcessInfo};

#[tauri::command]
pub fn process_get(show_all: bool) -> Vec<ProcessInfo> {
    process::get_process_list(show_all)
}
