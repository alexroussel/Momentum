//! `--smoke-test` mode (ADR-019): boot, migrate an in-memory DB, render the UI, exit 0.

use std::time::Duration;
use tauri::AppHandle;

const TIMEOUT: Duration = Duration::from_secs(90);

pub fn arm_timeout(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(TIMEOUT);
        eprintln!("smoke test: frontend did not report ready within {TIMEOUT:?}");
        app.exit(1);
    });
}

pub fn succeed(app: &AppHandle) {
    println!("smoke test: ok");
    app.exit(0);
}
