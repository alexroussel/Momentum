//! Per-OS window styling (ADR-011).

use crate::state::AppState;
use tauri::{Manager, WebviewWindow};

/// Applies the native window material where available and records whether it worked,
/// so the frontend can switch to a translucent background.
pub fn apply_platform_style(window: &WebviewWindow) {
    let applied = apply_effects(window);
    if let Some(state) = window.try_state::<AppState>() {
        if let Ok(mut v) = state.vibrancy.lock() {
            *v = applied;
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_effects(window: &WebviewWindow) -> bool {
    use tauri::utils::config::WindowEffectsConfig;
    use tauri::window::Effect;
    window
        .set_effects(WindowEffectsConfig {
            effects: vec![Effect::Mica],
            ..Default::default()
        })
        .is_ok()
        && is_windows_11()
}

#[cfg(target_os = "windows")]
fn is_windows_11() -> bool {
    // Mica is only rendered on Windows 11 (build 22000+); Windows 10 accepts the call silently.
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("cmd")
        .args(["/C", "ver"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.split('.')
                .nth(2)
                .and_then(|b| b.trim_end_matches(']').trim().parse::<u32>().ok())
        })
        .is_some_and(|build| build >= 22000)
}

#[cfg(target_os = "macos")]
fn apply_effects(window: &WebviewWindow) -> bool {
    use tauri::utils::config::WindowEffectsConfig;
    use tauri::window::{Effect, EffectState};
    window
        .set_effects(WindowEffectsConfig {
            effects: vec![Effect::Sidebar],
            state: Some(EffectState::FollowsWindowActiveState),
            ..Default::default()
        })
        .is_ok()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn apply_effects(_window: &WebviewWindow) -> bool {
    false
}
