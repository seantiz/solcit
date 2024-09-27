use tauri::AppHandle;
user serde_json::json;
use std::fs;

#[tauri::command]
pub fn initialise_config(app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    let files = vec![
        ("config.json", json!({"cvFilename":"","coverLetter":""})),
        ("jobDescription.json", json!({
            "jobTitle": "",
            "company": "",
            "jobDescription": "",
            "keyRequirements": "",
            "name": "",
            "experience": "",
            "interests": "",
            "projects": "",
            "education": "",
            "certificates": ""
        })),
        ("applicant_details.json", json!({
            "name": "",
            "experience": "",
            "interests": "",
            "projects": "",
            "education": "",
            "certificates": ""
        })),
    ];

    for (file_name, initial_content) in files {
        let file_path = app_dir.join(file_name);
        if !file_path.exists() {
            let content = serde_json::to_string_pretty(&initial_content)
                .map_err(|e| e.to_string())?;
            fs::write(file_path, content).map_err(|e| e.to_string())?;
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
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    fs::read_to_string(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_job_description(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    fs::write(file_path, content).map_err(|e| e.to_string())
}