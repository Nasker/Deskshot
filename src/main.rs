//! Command-line front-end.

mod cli;

use std::process::ExitCode;

use cli::Command;
use deskshot::composition;

fn main() -> ExitCode {
    let Some(command) = Command::parse(std::env::args().skip(1)) else {
        eprintln!("{}", cli::USAGE);
        return ExitCode::from(2);
    };
    let outcome = match command {
        Command::Capture(path) => composition::capture(&path).map(|s| cli::print_capture(&s)),
        Command::Restore(path) => composition::restore(&path).map(|r| cli::print_restore(&r)),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
