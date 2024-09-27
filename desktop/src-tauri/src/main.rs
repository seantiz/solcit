#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod llm;
mod server;
mod schemas;
mod helpers;
mod appconfig;
mod jobsearch;

use tauri::Manager;
use log::info;
use rusqlite::Connection;
use std::fs::{self, File};
use window_shadows::set_shadow;
use simplelog::{LevelFilter, CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode};

use llm::{suggestions, extract_cv};
use jobsearch::{find_indeed_listings, find_jooble_listings};
use server::{start_python_server, initialise_database, get_unread_jobs, update_job, get_stats};
use appconfig::{initialise_config, read_config, write_config, write_job_description, read_job_description};
use helpers::{get_db_path, get_log_file_path};

fn main() {
    let log_file = get_log_file_path();

    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(&log_file).expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to initialize logger");

    info!("Logging initialized at {:?}", log_file);

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");

            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let db_path = get_db_path(&app.handle());

            fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

            let conn = Connection::open(&db_path).expect("Failed to open or create database");
            initialise_database(&conn).unwrap();
            initialise_config(app.handle()).unwrap();

            Ok::<(), Box<dyn std::error::Error>>(())
        })
        .invoke_handler(tauri::generate_handler![
            find_indeed_listings,
            find_jooble_listings,
            write_job_description,
            read_job_description,
            initialise_config,
            write_config,
            read_config,
            start_python_server,
            get_unread_jobs,
            update_job,
            get_stats,
            suggestions,
            extract_cv,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn quit_app() {
    std::process::exit(0);
}
