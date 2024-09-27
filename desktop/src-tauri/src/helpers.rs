use tokio::process::Command as TokioCommand;
use simplelog::{info, error};
use tauri::AppHandle;
use std::path::PathBuf;
use std::future::Future;
use std::pin::Pin;
use home;

pub fn get_resource_path(app_handle: &AppHandle, resource: &str) -> PathBuf {
    app_handle.path_resolver()
        .resolve_resource(format!("resources/{}", resource))
        .expect(&format!("failed to resolve resource {}", resource))
}

pub fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        // Development mode
        get_resource_path(app_handle, "insegnante.sqlite")
    } else {
        // Production mode
        app_handle.path_resolver().app_data_dir()
            .unwrap()
            .join("insegnante.sqlite")
    }
}

pub fn get_log_file_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        // On macOS, use ~/Library/Logs/YourAppName/
        home::home_dir()
            .expect("Failed to get home directory")
            .join("Library/Logs/Solicit/app.log")
    } else {
        // For other OS, use the current executable's directory
        std::env::current_exe()
            .expect("Failed to get current exe path")
            .parent()
            .expect("Failed to get parent directory")
            .join("app.log")
    }
}

pub async fn execute_command(cmd: &str) -> Result<String, String> {
    info!("Executing command: {}", cmd);
    let output = TokioCommand::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to execute command: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    info!("TokioCommand STDOUT: {}", stdout);
    info!("TokioCommand STDERR: {}", stderr);

    if !output.status.success() {
        let error_msg = format!("TokioCommand failed: {}. STDERR: {}", cmd, stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    info!("TokioCommand executed successfully");
    Ok(stdout)
}

pub fn copy_directory(src: PathBuf, dst: PathBuf) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>> {
    Box::pin(async move {
        info!("Copying directory from {:?} to {:?}", src, dst);
        tokio::fs::create_dir_all(&dst).await?;
        let mut entries = tokio::fs::read_dir(&src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let ty = entry.file_type().await?;
            if ty.is_dir() {
                copy_directory(entry.path(), dst.join(entry.file_name())).await?;
            } else {
                tokio::fs::copy(entry.path(), dst.join(entry.file_name())).await?;
                info!("Copied file: {:?}", entry.path());
            }
        }
        info!("Directory copied successfully");
        Ok(())
    })
}