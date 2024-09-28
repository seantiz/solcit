use tauri::AppHandle;
use serde_json::json;
use log::{info, error};
use std::fs;

pub fn initialise_config(app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");

    fs::create_dir_all(&app_dir).map_err(|e| {
        error!("Failed to create app data directory: {}", e);
        e.to_string()
    })?;

    let files = vec![
        ("config.json", json!({"cvFilename":"","coverLetter":""})),
        ("jobDescription.json", json!({
            "jobTitle": "",
            "company": "",
            "jobDescription": "",
            "keyRequirements": "",
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
            info!("Creating first user config files: {:?}", file_path);
            let content = serde_json::to_string_pretty(&initial_content)
                .map_err(|e| {
                    error!("Failed to serialize JSON for {}: {}", file_name, e);
                    e.to_string()
                })?;
            fs::write(&file_path, content).map_err(|e| {
                error!("Failed to write files {}: {}", file_name, e);
                e.to_string()
            })?;
        } else {
            info!("File already exists: {:?}", file_path);
        }
    }

    Ok(())
}


#[tauri::command]
pub fn write_config(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    match fs::write(&file_path, &content) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            error!("Failed to write config: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn read_config(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            Ok(content)
        },
        Err(e) => {
            error!("Failed to read config: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn read_job_description(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            Ok(content)
        },
        Err(e) => {
            error!("Failed to read job description: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn write_job_description(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    match fs::write(&file_path, &content) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            error!("Failed to write job description: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn read_applicant_details(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("applicant_details.json");
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            Ok(content)
        },
        Err(e) => {
            error!("Failed to read applicant details: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn write_applicant_details(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("applicant_details.json");
    match fs::write(&file_path, &content) {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            error!("Failed to write applicant details: {}", e);
            Err(e.to_string())
        }
    }
}
