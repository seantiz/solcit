use crate::schemas::{Job, JobUpdate, Stats};
use crate::helpers::get_db_path;
use rusqlite::Connection;
use tauri::async_runtime::spawn;
use log::{info, error};
use tauri::AppHandle;
use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc;
use std::io::{BufRead, BufReader};

pub fn start_api_server(app_handle:&AppHandle) -> Result<(), String> {
    let api_executable = app_handle.path_resolver()
        .resolve_resource("resources/startup/apistart")
        .ok_or_else(|| "Failed to resolve apistart resource".to_string())?;

    let api_executable_str = api_executable.to_str()
        .ok_or_else(|| "Failed to convert path to string".to_string())?
        .to_string();

    info!("Starting API server at: {}", api_executable_str);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        match Command::new(&api_executable_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                info!("API server process started successfully");

                let stdout = child.stdout.take()
                    .ok_or_else(|| "Failed to capture stdout".to_string())?;
                let stderr = child.stderr.take()
                    .ok_or_else(|| "Failed to capture stderr".to_string())?;

                let tx_stdout = tx.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            let _ = tx_stdout.send(format!("STDOUT: {}", line));
                            info!("API Server STDOUT: {}", line);
                        }
                    }
                });

                let tx_stderr = tx.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            let _ = tx_stderr.send(format!("STDERR: {}", line));
                            error!("API Server STDERR: {}", line);
                        }
                    }
                });

                match child.wait() {
                    Ok(status) => info!("API server process exited with status: {:?}", status),
                    Err(e) => error!("Error waiting for API server process: {}", e),
                }
            }
            Err(e) => {
                return Err(format!("Failed to start API server: {}", e));
            }
        }
        Ok(())
    });

    thread::spawn(move || {
        for received in rx {
            info!("API Server output: {}", received);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn get_unread_jobs(app_handle:AppHandle) -> Result<Vec<Job>, String> {
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
pub fn update_job(app_handle:AppHandle, job_id: i32, job_update: JobUpdate) -> Result<Job, String> {
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
pub async fn get_stats(app_handle:AppHandle) -> Result<Stats, String> {
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
