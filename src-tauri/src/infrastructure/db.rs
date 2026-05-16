use std::{fs, path::PathBuf};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use super::repositories::SqliteRepository;

const DATABASE_FILE_NAME: &str = "focus-tray.sqlite3";

pub fn initialize(app: &AppHandle) -> Result<SqliteRepository, String> {
    let database_path = database_path(app)?;
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create app data directory: {error}"))?;
    }

    let connection = Connection::open(&database_path).map_err(|error| {
        format!(
            "failed to open SQLite database at `{}`: {error}",
            database_path.display()
        )
    })?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("failed to enable SQLite foreign keys: {error}"))?;

    let repository = SqliteRepository::new(connection);
    repository.run_migrations()?;
    Ok(repository)
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data directory: {error}"))?
        .join(DATABASE_FILE_NAME))
}
