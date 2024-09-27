use tauri::AppHandle;
use std::fs;

#[tauri::command]
pub fn initialise_config(app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let files = vec![
        ("config.json", r#"{"cvFilename":"","coverLetter":""}"#),
        ("jobDescription.json", "{}"),
        ("applicant_details.json", "{}"),
    ];

    for (file_name, initial_content) in files {
        let file_path = app_dir.join(file_name);
        if !file_path.exists() {
            fs::write(file_path, initial_content).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn write_config(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    fs::write(file_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_config(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    fs::read_to_string(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_job_description() -> Result<String, String> {
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default()).expect("Failed to get app dir");
    let file_path = app_data_dir.join("jobDescription.json");
    fs::read_to_string(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_job_description(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    fs::write(file_path, content).map_err(|e| e.to_string())
}