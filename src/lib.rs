//! deskshot: snapshot and restore window layouts across virtual desktops.
//!
//! `domain` holds the model and ports, `app` the use cases, `infra` the
//! platform adapters, and `composition` wires them together for the
//! front-ends in `main.rs` (CLI) and `bin/deskshot-gui.rs`.

pub mod app;
pub mod composition;
pub mod domain;
pub mod infra;
