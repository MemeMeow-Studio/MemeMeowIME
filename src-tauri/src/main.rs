// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use env_logger::Env;
use log::{debug, error, info, trace, warn};
use tauri::{Manager, Window};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    mememeow_tauri_lib::run()
}
