use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeskshotError {
    #[error("platform call failed: {0}")]
    Platform(String),

    #[error("virtual desktop operation failed: {0}")]
    VirtualDesktop(String),

    #[error("could not launch {app}: {reason}")]
    Launch { app: String, reason: String },

    #[error("snapshot storage failed: {0}")]
    Storage(String),

    #[cfg_attr(windows, allow(dead_code))] // only raised by the non-Windows stubs
    #[error("{0} is not supported on this platform")]
    Unsupported(&'static str),
}

pub type Result<T> = std::result::Result<T, DeskshotError>;
