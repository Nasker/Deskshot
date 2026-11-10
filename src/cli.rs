//! Argument parsing and console output. No business logic.

use std::path::PathBuf;

use deskshot::app::restore::{RestoreFailure, RestoreReport};
use deskshot::domain::model::Snapshot;

pub const USAGE: &str = "usage: deskshot <capture|restore> <snapshot.json>";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Capture(PathBuf),
    Restore(PathBuf),
}

impl Command {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Option<Self> {
        let verb = args.next()?;
        let path = PathBuf::from(args.next()?);
        if args.next().is_some() {
            return None;
        }
        match verb.as_str() {
            "capture" => Some(Self::Capture(path)),
            "restore" => Some(Self::Restore(path)),
            _ => None,
        }
    }
}

pub fn print_capture(snapshot: &Snapshot) {
    println!(
        "Captured {} windows across {} desktops.",
        snapshot.windows.len(),
        snapshot.desktop_count
    );
    for window in &snapshot.windows {
        let desktop = window.desktop.map_or("-".to_string(), |d| (d + 1).to_string());
        println!("  [desktop {desktop}] {} — {}", window.identity.app, window.identity.title);
    }
}

pub fn print_restore(report: &RestoreReport) {
    println!("Restored {} windows.", report.restored);
    print_failures("Could not launch", &report.launch_failures);
    print_failures("Could not place", &report.placement_failures);
    for window in &report.missing {
        println!("  Never appeared: {} — {}", window.identity.app, window.identity.title);
    }
}

fn print_failures(label: &str, failures: &[RestoreFailure]) {
    for failure in failures {
        println!("  {label} {}: {}", failure.window.identity.title, failure.error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Option<Command> {
        Command::parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_both_verbs() {
        assert_eq!(parse(&["capture", "a.json"]), Some(Command::Capture("a.json".into())));
        assert_eq!(parse(&["restore", "a.json"]), Some(Command::Restore("a.json".into())));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(parse(&["capture"]), None);
        assert_eq!(parse(&["explode", "a.json"]), None);
        assert_eq!(parse(&["capture", "a.json", "extra"]), None);
    }
}
