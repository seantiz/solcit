use crate::schemas::ParsedDetails;
use reqwest::Client;
use serde_json::json;

#[tauri::command]
pub async fn suggestions(query_details: serde_json::Value) -> Result<String, String> {
    let client = Client::new();
    let res = client.post("http://localhost:8080/api/suggestions")
        .json(&query_details)
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
pub async fn extract_cv(preprocessed_text: String) -> Result<ParsedDetails, String> {
    let client = Client::new();
    let res = client.post("http://localhost:8080/api/cv")
        .json(&json!({ "text": preprocessed_text }))
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