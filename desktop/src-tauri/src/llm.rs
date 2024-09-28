use crate::schemas::ParsedDetails;
use crate::helpers::get_credentials_file_path;
use reqwest::Client;
use serde_json::{json, Value};
use tauri::AppHandle;
use std::fs;
use log::info;

#[tauri::command]
pub fn get_key(app_handle: AppHandle) -> Result<String, String> {
    let file_path = get_credentials_file_path(&app_handle);

    if !file_path.exists() {
        return Ok("".to_string());
    }

    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read credentials file: {}", e))?;

    let json: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse credentials JSON: {}", e))?;

    Ok(json["anthropic_api_key"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
pub fn set_key(app_handle: AppHandle, key: String) -> Result<(), String> {
    let file_path = get_credentials_file_path(&app_handle);

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let json = json!({
        "anthropic_api_key": key
    });

    fs::write(file_path.clone(), serde_json::to_string_pretty(&json).unwrap())
        .map_err(|e| format!("Failed to write credentials file: {}", e))?;

    info!("Credentials file successfully written to: {}", file_path.display());

    Ok(())
}


#[tauri::command]
pub async fn suggestions(app_handle: AppHandle, query_details: serde_json::Value) -> Result<String, String> {
    let api_key = get_key(app_handle)?;
    let mut request_body = query_details;
    request_body["anthropic_api_key"] = serde_json::Value::String(api_key);

    let client = Client::new();
    let res = client.post("http://localhost:8080/api/suggestions")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    if !status.is_success() {
        let error_text = res.text().await.map_err(|e| e.to_string())?;
        return Err(format!("HTTP Error: {}, message: {}", status, error_text));
    }

    let response_json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let cover_letter = response_json["cover_letter"].as_str()
        .ok_or_else(|| "Cover letter not found in response".to_string())?
        .to_string();

    Ok(cover_letter)
}

#[tauri::command]
pub async fn extract_cv(app_handle: AppHandle, preprocessed_text: String) -> Result<ParsedDetails, String> {
    let api_key = get_key(app_handle)?;
    let request_body = json!({
        "text": preprocessed_text,
        "anthropic_api_key": api_key
    });

    let client = Client::new();
    let res = client.post("http://localhost:8080/api/cv")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    if !status.is_success() {
        let error_text = res.text().await.map_err(|e| e.to_string())?;
        return Err(format!("HTTP Error: {}, message: {}", status, error_text));
    }

    let backend_result: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    let frontend_result = ParsedDetails {
        experience: backend_result["Experience"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default(),
        interests: backend_result["Interests"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default(),
        projects: backend_result["Projects"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default(),
        education: backend_result["Education"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default(),
        certificates: backend_result["Certificates"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default(),
    };

    Ok(frontend_result)
}