//! Use cases. Everything here depends only on `domain` ports, so it is
//! platform-independent and unit-testable with fakes.

pub mod capture;
pub mod matching;
pub mod restore;

#[cfg(test)]
pub(crate) mod fakes;
