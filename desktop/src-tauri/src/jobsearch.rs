use crate::helpers::{get_resource_path, get_db_path};
use tauri::AppHandle;

#[tauri::command]
pub async fn find_indeed_listings(app_handle: AppHandle, keywords: String, location: String) -> Result<(), String> {
    let db_path = get_db_path(&app_handle);
    let find_listings = get_resource_path(&app_handle, "search_engine_indeed.py");

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
    let find_listings = get_resource_path(&app_handle, "search_engine_jooble.py");

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