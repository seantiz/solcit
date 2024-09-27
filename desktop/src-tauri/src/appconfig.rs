use tauri::AppHandle;
use std::path::PathBuf;
use serde_json::json;
use log::{info, error, debug};
use std::fs;

fn is_app_initialized(app_dir: &PathBuf) -> bool {
    let flag_file = app_dir.join("app_initialized.flag");
    let exists = flag_file.exists();
    debug!("Checking if app is initialized. Flag file: {:?}, Exists: {}", flag_file, exists);
    exists
}

fn first_init(app_dir: &PathBuf) -> Result<(), String> {
    info!("Performing first initialization");
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
            info!("Creating file: {:?}", file_path);
            let content = serde_json::to_string_pretty(&initial_content)
                .map_err(|e| {
                    error!("Failed to serialize JSON for {}: {}", file_name, e);
                    e.to_string()
                })?;
            fs::write(&file_path, content).map_err(|e| {
                error!("Failed to write file {}: {}", file_name, e);
                e.to_string()
            })?;
        } else {
            debug!("File already exists: {:?}", file_path);
        }
    }

    let flag_file = app_dir.join("app_initialized.flag");
    info!("Creating initialization flag file: {:?}", flag_file);
    fs::write(&flag_file, "").map_err(|e| {
        error!("Failed to create initialization flag file: {}", e);
        e.to_string()
    })?;

    info!("First initialization completed successfully");
    Ok(())
}

pub fn initialise_config(app_handle: AppHandle) -> Result<(), String> {
    info!("Initializing config");
    let app_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    info!("App data directory: {:?}", app_dir);

    fs::create_dir_all(&app_dir).map_err(|e| {
        error!("Failed to create app data directory: {}", e);
        e.to_string()
    })?;

    if !is_app_initialized(&app_dir) {
        info!("App not initialized, performing first init");
        first_init(&app_dir)?;
    } else {
        info!("App already initialized");
    }

    info!("Config initialization completed");
    Ok(())
}

#[tauri::command]
pub fn write_config(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    info!("Writing config to: {:?}", file_path);
    match fs::write(&file_path, &content) {
        Ok(_) => {
            info!("Successfully wrote config");
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
    info!("Reading config from: {:?}", file_path);
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            info!("Successfully read config");
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
    info!("Reading job description from: {:?}", file_path);
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            info!("Successfully read job description");
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
    info!("Writing job description to: {:?}", file_path);
    match fs::write(&file_path, &content) {
        Ok(_) => {
            info!("Successfully wrote job description");
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
    info!("Reading applicant details from: {:?}", file_path);
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            info!("Successfully read applicant details");
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
    info!("Writing applicant details to: {:?}", file_path);
    match fs::write(&file_path, &content) {
        Ok(_) => {
            info!("Successfully wrote applicant details");
            Ok(())
        },
        Err(e) => {
            error!("Failed to write applicant details: {}", e);
            Err(e.to_string())
        }
    }
}
