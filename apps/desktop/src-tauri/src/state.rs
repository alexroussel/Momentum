use momentum_store::{Store, DB_FILE_NAME};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Process-wide state managed by Tauri.
pub struct AppState {
    pub store: Mutex<Store>,
    pub smoke_test: bool,
    /// Whether a native window material (Mica / vibrancy) was applied.
    pub vibrancy: Mutex<bool>,
}

impl AppState {
    pub fn init(app: &AppHandle, smoke_test: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let store = if smoke_test {
            Store::open_in_memory()?
        } else {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            Store::open(&dir.join(DB_FILE_NAME))?
        };
        Ok(Self {
            store: Mutex::new(store),
            smoke_test,
            vibrancy: Mutex::new(false),
        })
    }
}
