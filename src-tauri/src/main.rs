// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod operations {
    pub mod process;
    pub mod profile;
    pub mod user_settings;
    pub mod window_manager;
    pub mod window_state;
}

mod setup {
    pub mod events;
    pub mod init;
    pub mod shortcuts;
    pub mod state;
    pub mod tray;
    pub mod ipc;
}

mod commands;
mod errors;


mod debug_utils;

use crate::operations::{process, profile, user_settings, window_manager};

fn is_already_running() -> bool {
    use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::synchapi::CreateMutexW;

    let name: Vec<u16> = "ResizeRabbitSingleInstance\0".encode_utf16().collect();
    // Intentionally not closing this handle — it must stay open for the mutex to exist.
    let _handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()) };
    let last_error = unsafe { GetLastError() };
    last_error == ERROR_ALREADY_EXISTS
}

fn main() {
    dotenvy::dotenv().ok();

    if is_already_running() {
        // Tell the running instance to show its window, then exit.
        if let Ok(mut pipe) = std::fs::OpenOptions::new()
            .write(true)
            .open(r"\\.\pipe\resize-rabbit")
        {
            use std::io::Write;
            let _ = pipe.write_all(b"show");
        }
        return;
    }

    let builder = tauri::Builder::default();
    let builder = commands::register_commands(builder);
    let builder = setup::init::setup(builder);
    let builder = setup::tray::setup_tray(builder);
    let builder = setup::events::handle_events(builder);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
