//! Platform-agnostic core: the data model and the ports (traits) that the
//! application layer depends on. Nothing in here knows about Win32.

pub mod error;
pub mod model;
pub mod ports;
