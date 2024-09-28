#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod llm;
mod server;
mod schemas;
mod helpers;
mod appconfig;
mod jobsearch;

use tauri::Manager;
use rusqlite::Connection;
use std::fs::{self, File};
use std::path::Path;
use window_shadows::set_shadow;
use simplelog::{LevelFilter, CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode};

use llm::{set_key, get_key, suggestions, extract_cv};
use jobsearch::{find_indeed_listings, run_jooble_search};
use server::{start_api_server, get_unread_jobs, update_job, get_stats};
use appconfig::{initialise_config, read_config, write_config, write_job_description, read_job_description, read_applicant_details, write_applicant_details};
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
            LevelFilter::Debug,
            Config::default(),
            File::create(&log_file).expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to initialize logger");

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");

            let app_dir = app.path_resolver().app_data_dir().unwrap();
            fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

            copy_database(app)?;

            let db_path = get_db_path(&app.handle());
            let _conn = Connection::open(&db_path).expect("Failed to open database");

            initialise_config(app.handle()).unwrap();

            // Start the API server
            start_api_server(&app.handle()).expect("Failed to start API server");

            Ok::<(), Box<dyn std::error::Error>>(())
        })
        .invoke_handler(tauri::generate_handler![
            find_indeed_listings,
            run_jooble_search,
            write_job_description,
            read_job_description,
            write_applicant_details,
            read_applicant_details,
            write_config,
            read_config,
            get_unread_jobs,
            update_job,
            get_stats,
            get_key,
            set_key,
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

fn copy_database(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = app.path_resolver()
        .resolve_resource("resources/insegnante.sqlite")
        .expect("Failed to resolve source database path");

    let dest_path = get_db_path(&app.handle());

    if !Path::new(&dest_path).exists() {
        fs::copy(source_path, &dest_path)?;
        println!("Database copied from resources to: {:?}", dest_path);
    } else {
        println!("Database already exists at: {:?}", dest_path);
    }

    Ok(())
}
