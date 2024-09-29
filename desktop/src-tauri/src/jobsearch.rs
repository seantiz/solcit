use crate::helpers::get_db_path;
use std::process::Command;
use log::{info, error};
use tauri::AppHandle;
use tokio::task;

#[tauri::command]
pub async fn run_indeed_search(app_handle: AppHandle, keywords: String, location: String) -> Result<String, String> {

    let db_path = get_db_path(&app_handle);
    let indeed_executable = app_handle
        .path_resolver()
        .resolve_resource("resources/indeedsearchengine")
        .expect("Failed to resolve Indeed search engine executable path");
    info!("Indeed executable path: {:?}", indeed_executable);

    let result = task::spawn_blocking(move || {
        info!("Executing Indeed search engine...");
        let output = Command::new(&indeed_executable)
            .arg(db_path.to_str().unwrap())
            .arg(&keywords)
            .arg(&location)
            .output()
            .map_err(|e| {
                let error_msg = format!("Failed to execute Indeed search engine: {}", e);
                error!("{}", error_msg);
                error_msg
            })?;

        if !output.status.success() {
            let error_msg = format!("Indeed search engine execution failed. Exit code: {:?}\nStderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr));
            error!("{}", error_msg);
            return Err(error_msg);
        }

        let success_msg = format!("Indeed search completed successfully. Stdout: {}",
            String::from_utf8_lossy(&output.stdout));
        info!("{}", success_msg);
        Ok(success_msg)
    }).await.map_err(|e| format!("Task join error: {}", e))?;

    result
}

#[tauri::command]
pub async fn run_jooble_search(app_handle: AppHandle, keywords: String, location: String) -> Result<String, String> {

    let db_path = get_db_path(&app_handle);
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

