use tauri::AppHandle;
use std::path::PathBuf;

pub fn get_db_path(app_handle: &AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        // Use get_system_resource_path callback in prod mode
        /* User-related info best stored in tauri.AppHandle.app.data.dir() in Prod
        macOS : ~/Library/Application Support/<AppName>/dbfilename.sqlite
        Windows: C:\Users\<Username>\AppData\Roaming\<AppName>\dbfilename.sqlite
        Linux: ~/.local/share/<AppName>/dbfilename.sqlite */

        get_system_resource_path(app_handle, "insegnante.sqlite")
    } else {
        // Dev: src-tauri/resources/dbfilename.sqlite
        get_user_resource_path(app_handle, "insegnante.sqlite")
    }
}

fn get_system_resource_path(app_handle: &AppHandle, resource: &str) -> PathBuf {
    // Dev: src-tauri/resources/<resource>

    /* Prod
    Windows: C:\Program Files\<Appname>\resources\<resource>
    macOS: /Applications/<Appname>.app/Contents/Resources/<resource>
    Linux /usr/share/<appname>/resources/<resource> or ~/.local/share/<appname>/resources/<resource>
    */
    app_handle.path_resolver()
        .resolve_resource(format!("resources/{}", resource))
        .expect(&format!("failed to resolve resource {}", resource))
}

fn get_user_resource_path(app_handle: &AppHandle, resource: &str) -> PathBuf {
    // Local helper for get_db_path only
    let app_local_data_dir = app_handle.path_resolver()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    app_local_data_dir.join(resource)
}