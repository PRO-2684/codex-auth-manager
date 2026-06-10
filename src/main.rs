#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod cli;

use cli::CliError;
use std::process::ExitCode;

const EXIT_OK: u8 = 0;
const EXIT_RUNTIME_ERROR: u8 = 1;
const EXIT_USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    cli::complete_from_env();

    match cli::run() {
        Ok(()) => ExitCode::from(EXIT_OK),
        Err(CliError::Usage(error)) => {
            let _ = error.print();
            ExitCode::from(EXIT_USAGE_ERROR)
        }
        Err(CliError::Runtime(error)) => {
            eprintln!("{error}");
            ExitCode::from(EXIT_RUNTIME_ERROR)
        }
    }
}
