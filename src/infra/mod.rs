//! Adapters implementing the domain ports.

pub mod json_store;
pub mod process_launcher;

#[cfg(windows)]
pub mod windows;
