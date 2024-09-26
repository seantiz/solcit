#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::PathBuf;
use std::fs;
use std::pin::Pin;
use std::future::Future;
use tauri::Manager;
use window_shadows::set_shadow;
use rusqlite::{Connection, Result as SqliteResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::async_runtime::spawn;
use tauri::AppHandle;
use log::{info, error, LevelFilter};
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode};
use std::fs::File;
use home;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};
use std::process::Stdio;


#[derive(Debug, Serialize, Deserialize)]
struct Job {
    id: i32,
    uniqueid: String,
    title: String,
    company: String,
    location: String,
    salary: String,
    jobkey: String,
    fetched_date: String,
    read: bool,
    appliedto: bool,
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobUpdate {
    read: Option<bool>,
    appliedto: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    uniquejobs: i32,
    appliedjobs: i32,
}

#[derive(Serialize)]
struct ParsedDetails {
    experience: String,
    interests: String,
    projects: String,
    education: String,
    certificates: String,
}

fn main() {
    // Setup logging
    let log_file = get_log_file_path();

    // Ensure the directory exists
    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(&log_file).expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to initialize logger");

    info!("Logging initialized at {:?}", log_file);

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");

            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let db_path = get_db_path(&app.handle());

            fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

            let conn = Connection::open(&db_path).expect("Failed to open or create database");
            initialise_database(&conn).unwrap();
            initialise_config(app.handle()).unwrap();

            Ok::<(), Box<dyn std::error::Error>>(())
        })
        .invoke_handler(tauri::generate_handler![
            start_python_server,
            find_indeed_listings,
            find_jooble_listings,
            get_stats,
            get_unread_jobs,
            update_job,
            suggestions,
            extract_cv_details,
            quit_app,
            initialise_config,
            write_config_file,
            read_config_file,
            write_job_description,
            read_job_description
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_log_file_path() -> PathBuf {
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

fn get_resource_path(app_handle: &AppHandle, resource: &str) -> PathBuf {
    app_handle.path_resolver()
        .resolve_resource(format!("resources/{}", resource))
        .expect(&format!("failed to resolve resource {}", resource))
}

fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
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

#[tauri::command]
fn quit_app() {
    std::process::exit(0);
}

#[tauri::command]
fn initialise_database(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY,
            uniqueid TEXT,
            title TEXT,
            company TEXT,
            location TEXT,
            salary TEXT,
            jobkey TEXT,
            fetched_date TEXT,
            read BOOLEAN,
            appliedto BOOLEAN,
            source TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS stats (
            id INTEGER PRIMARY KEY,
            uniquejobs INTEGER,
            appliedjobs INTEGER
        )",
        [],
    )?;

    Ok(())
}

#[tauri::command]
fn initialise_config(app_handle: AppHandle) -> Result<(), String> {
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
fn write_config_file(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    fs::write(file_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_config_file(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("config.json");
    fs::read_to_string(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_job_description() -> Result<String, String> {
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default()).expect("Failed to get app dir");
    let file_path = app_data_dir.join("jobDescription.json");
    std::fs::read_to_string(file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_job_description(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app data dir");
    let file_path = app_data_dir.join("jobDescription.json");
    fs::write(file_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_python_server(app_handle: AppHandle) -> Result<(), String> {
    let is_dev = cfg!(debug_assertions);
    info!("Is development mode: {}", is_dev);

    let app_local_data_dir = app_handle.path_resolver()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    info!("App local data directory: {:?}", app_local_data_dir);

    let resources_path = if is_dev {
        app_handle.path_resolver()
            .resolve_resource("resources")
            .expect("Failed to resolve resources directory")
    } else {
        app_local_data_dir.join("resources")
    };
    info!("Resources path: {:?}", resources_path);

    let api_path = resources_path.join("api.py");
    let requirements_path = resources_path.join("requirements.txt");
    let venv_path = if is_dev {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..").join("..").join("backend").join("venv")
    } else {
        resources_path.join("venv")
    };
    let venv_activate = venv_path.join("bin").join("activate");

    info!("API path: {:?}", api_path);
    info!("Requirements path: {:?}", requirements_path);
    info!("Venv path: {:?}", venv_path);
    info!("Venv activate path: {:?}", venv_activate);

    if !is_dev {
        info!("Running in production mode, setting up environment...");

        // Copy resources to app local data directory if they don't exist
        if !resources_path.exists() {
            info!("Resources directory doesn't exist, creating and copying...");
            let bundled_resources = app_handle.path_resolver()
                .resolve_resource("resources")
                .expect("Failed to resolve bundled resources");
            info!("Bundled resources path: {:?}", bundled_resources);

            tokio::fs::create_dir_all(&resources_path).await
                .map_err(|e| format!("Failed to create resources directory: {}", e))?;

            copy_directory(bundled_resources, resources_path).await
                .map_err(|e| format!("Failed to copy resources: {}", e))?;
        } else {
            info!("Resources directory already exists");
        }

        // Create virtual environment
        info!("Creating virtual environment...");
        let create_venv_cmd = format!("python3 -m venv {:?}", venv_path);
        execute_command(&create_venv_cmd).await?;

        // Activate virtual environment and install requirements
        info!("Activating virtual environment and installing requirements...");
        let install_req_cmd = format!("source {:?} && pip install -r {:?}", venv_activate, requirements_path);
        execute_command(&install_req_cmd).await?;
    }

    // Run the API script as a background process
    info!("Starting the Python server...");
    let run_api_cmd = format!("source {:?} && python3 {:?}", venv_activate, api_path);
    TokioCommand::new("sh")
        .arg("-c")
        .arg(&run_api_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python server: {}", e))?;

    // Wait a bit to allow the server to start
    sleep(Duration::from_secs(2)).await;

    let mut child = TokioCommand::new("sh")
    .arg("-c")
    .arg(&run_api_cmd)
    .spawn()
    .map_err(|e| {
        error!("Failed to start Python server: {}", e);
        format!("Failed to start Python server: {}", e)
    })?;

// Check if the process is still running
match child.try_wait() {
    Ok(Some(status)) => {
        error!("Python server exited unexpectedly with status: {}", status);
        Err("Python server exited unexpectedly".to_string())
    }
    Ok(None) => {
        info!("Python server started successfully and is running");
        Ok(())
    }
    Err(e) => {
        error!("Error checking Python server status: {}", e);
        Err(format!("Error checking Python server status: {}", e))
    }
}
}

async fn execute_command(cmd: &str) -> Result<String, String> {
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


fn copy_directory(src: PathBuf, dst: PathBuf) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>> {
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

#[tauri::command]
fn update_job(app_handle: tauri::AppHandle, job_id: i32, job_update: JobUpdate) -> Result<Job, String> {
    let app_data_dir = app_handle.path_resolver().app_data_dir().unwrap();
    let db_path = app_data_dir.join("insegnante.sqlite");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    if let Some(read) = job_update.read {
        conn.execute(
            "UPDATE jobs SET read = ?1 WHERE id = ?2",
            [read as i32, job_id],
        ).map_err(|e| e.to_string())?;
    }

    if let Some(appliedto) = job_update.appliedto {
        conn.execute(
            "UPDATE jobs SET appliedto = ?1, read = 1 WHERE id = ?2",
            [appliedto as i32, job_id],
        ).map_err(|e| e.to_string())?;

        if appliedto {
            conn.execute(
                "UPDATE stats SET appliedjobs = appliedjobs + 1 WHERE id = 1",
                [],
            ).map_err(|e| e.to_string())?;
        }
    }

    let job = conn.query_row(
        "SELECT * FROM jobs WHERE id = ?1",
        [job_id],
        |row| {
            Ok(Job {
                id: row.get(0)?,
                uniqueid: row.get(1)?,
                title: row.get(2)?,
                company: row.get(3)?,
                location: row.get(4)?,
                salary: row.get(5)?,
                jobkey: row.get(6)?,
                fetched_date: row.get(7)?,
                read: row.get(8)?,
                appliedto: row.get(9)?,
                source: row.get(10)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(job)
}

#[tauri::command]
async fn find_indeed_listings(app_handle: tauri::AppHandle, keywords: String, location: String) -> Result<(), String> {
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
async fn find_jooble_listings(app_handle: tauri::AppHandle, keywords: String, location: String) -> Result<(), String> {
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

#[tauri::command]
async fn get_stats(app_handle: tauri::AppHandle) -> Result<Stats, String> {
    let stats = spawn(async move {
        let db_path = get_db_path(&app_handle);
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare("SELECT uniquejobs, appliedjobs FROM stats WHERE id = 1")
            .map_err(|e| e.to_string())?;

        let stats = stmt.query_row([], |row| {
            Ok(Stats {
                uniquejobs: row.get(0)?,
                appliedjobs: row.get(1)?,
            })
        }).unwrap_or(Stats { uniquejobs: 0, appliedjobs: 0 });

        Ok::<_, String>(stats)
    }).await;

    stats.map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn get_unread_jobs(app_handle: tauri::AppHandle) -> Result<Vec<Job>, String> {
    let jobs = spawn(async move {
        let db_path = get_db_path(&app_handle);
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT id, uniqueid, title, company, location, salary, jobkey, fetched_date, read, appliedto, source
             FROM jobs
             WHERE read = 0
             ORDER BY fetched_date DESC"
        ).map_err(|e| e.to_string())?;

        let jobs = stmt.query_map([], |row| {
            Ok(Job {
                id: row.get(0)?,
                uniqueid: row.get(1)?,
                title: row.get(2)?,
                company: row.get(3)?,
                location: row.get(4)?,
                salary: row.get(5)?,
                jobkey: row.get(6)?,
                fetched_date: row.get(7)?,
                read: row.get(8)?,
                appliedto: row.get(9)?,
                source: row.get(10)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok::<_, String>(jobs)
    }).await;

    jobs.map_err(|e| format!("Task join error: {}", e))?
}


#[tauri::command]
async fn suggestions(query_details: serde_json::Value) -> Result<String, String> {
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
async fn extract_cv_details(preprocessed_text: String) -> Result<ParsedDetails, String> {
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