//! Momentum core: domain types shared by the desktop app, the store and (later) the CLI.
//!
//! This crate deliberately has no Tauri dependency so that the timer state machine and the
//! analytics can be unit-tested quickly and reused by other binaries.

pub mod domain;
pub mod id;

pub use domain::*;
pub use id::new_id;
