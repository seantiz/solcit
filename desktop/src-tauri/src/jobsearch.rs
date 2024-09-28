use crate::helpers::{get_system_resource_path, get_db_path};
use std::process::Command;
use log::{info, error};
use tauri::AppHandle;
use tauri::Manager;
use tokio::task;

#[tauri::command]
pub async fn find_indeed_listings(app_handle: AppHandle, keywords: String, location: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let find_listings = get_system_resource_path(&app_handle, "search_engine_indeed.py");

    let output = std::process::Command::new("python3")
        .arg(&find_listings)
        .arg(db_path.to_str().unwrap())
        .arg(&keywords)
        .arg(&location)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        println!("Python script executed successfully");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("Python script failed: {}", error);
        Err(error.into_owned())
    }
}

#[tauri::command]
pub async fn find_jooble_listings(app_handle: AppHandle, keywords: String, location: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let find_listings = get_system_resource_path(&app_handle, "search_engine_jooble.py");

    let output = std::process::Command::new("python3")
        .arg(&find_listings)
        .arg(db_path.to_str().unwrap())
        .arg(&keywords)
        .arg(&location)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        println!("Python script executed successfully");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("Python script failed: {}", error);
        Err(error.into_owned())
    }
}

#[tauri::command]
pub async fn run_jooble_search(app_handle: AppHandle, keywords: String, location: String) -> Result<String, String> {
    info!("Starting Jooble search with keywords: '{}', location: '{}'", keywords, location);

    let db_path = get_db_path(&app_handle);
    info!("Database path: {:?}", db_path);

    let jooble_executable = app_handle
        .path_resolver()
        .resolve_resource("resources/joobsearchengine")
        .expect("Failed to resolve Jooble search engine executable path");
    info!("Jooble executable path: {:?}", jooble_executable);

    // Run the search in a separate thread
    let result = task::spawn_blocking(move || {
        info!("Executing Jooble search engine...");
        let output = Command::new(&jooble_executable)
            .arg(db_path.to_str().unwrap())
            .arg(&keywords)
            .arg(&location)
            .output()
            .map_err(|e| {
                let error_msg = format!("Failed to execute Jooble search engine: {}", e);
                error!("{}", error_msg);
                error_msg
            })?;

        if !output.status.success() {
            let error_msg = format!("Jooble search engine execution failed. Exit code: {:?}\nStderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr));
            error!("{}", error_msg);
            return Err(error_msg);
        }

        let success_msg = format!("Jooble search completed successfully. Stdout: {}",
            String::from_utf8_lossy(&output.stdout));
        info!("{}", success_msg);
        Ok(success_msg)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    result
}

