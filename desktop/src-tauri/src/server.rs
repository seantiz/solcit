use crate::schemas::{Job, JobUpdate, Stats};
use crate::helpers::{execute_command, copy_directory, get_db_path, get_system_resource_path};
use tokio::process::Command as TokioCommand;
use rusqlite::{Connection, Result as DBResult};
use tauri::async_runtime::spawn;
use log::{info, error};
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn start_python_server(app_handle: AppHandle) -> Result<(), String> {
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
        get_system_resource_path(&app_handle, "")
    };
    info!("Resources path: {:?}", resources_path);

    let api_path = if is_dev {
        resources_path.join("api.py")
    } else {
        get_system_resource_path(&app_handle, "api.py")
    };
    let requirements_path = if is_dev {
        resources_path.join("requirements.txt")
    } else {
        get_system_resource_path(&app_handle, "requirements.txt")
    };

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
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Python server: {}", e))?;

    // Wait a bit to allow the server to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

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

#[tauri::command]
pub fn initialise_database(conn: &Connection) -> DBResult<()> {
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
pub async fn get_unread_jobs(app_handle: tauri::AppHandle) -> Result<Vec<Job>, String> {
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
pub fn update_job(app_handle: tauri::AppHandle, job_id: i32, job_update: JobUpdate) -> Result<Job, String> {
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
pub async fn get_stats(app_handle: tauri::AppHandle) -> Result<Stats, String> {
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
