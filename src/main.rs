#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use std::{env, process::ExitCode, str::FromStr};

use codex_auth_manager::{
    AuthStatus, CaptureOptions, CodexAuthManager, DetachOptions, Error, IdentityName, UseOptions,
};

const EXIT_OK: u8 = 0;
const EXIT_RUNTIME_ERROR: u8 = 1;
const EXIT_USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(EXIT_OK),
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            ExitCode::from(EXIT_USAGE_ERROR)
        }
        Err(CliError::Runtime(error)) => {
            eprintln!("{error}");
            ExitCode::from(EXIT_RUNTIME_ERROR)
        }
    }
}

fn run() -> Result<(), CliError> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = parse_command(&args)?;
    match command {
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Status => {
            let manager = CodexAuthManager::from_env()?;
            println!("{}", manager.status()?);
            Ok(())
        }
        Command::List => {
            let manager = CodexAuthManager::from_env()?;
            for identity in manager.list()? {
                let marker = if identity.active { "*" } else { " " };
                let broken = if identity.broken { " (broken)" } else { "" };
                println!("{marker} {}{broken}", identity.name);
            }
            Ok(())
        }
        Command::Capture { identity, force } => {
            let manager = CodexAuthManager::from_env()?;
            manager.capture(&identity, CaptureOptions { force })?;
            println!("Captured identity: {identity}");
            Ok(())
        }
        Command::Use { identity, force } => {
            let manager = CodexAuthManager::from_env()?;
            manager.use_identity(&identity, UseOptions { force })?;
            println!("Active identity: {identity}");
            Ok(())
        }
        Command::Detach { force } => {
            let manager = CodexAuthManager::from_env()?;
            let status = manager.status()?;
            manager.detach(DetachOptions { force })?;
            match status {
                AuthStatus::Managed { identity } => {
                    println!("Detached from active identity: {identity}");
                }
                AuthStatus::BrokenManaged { identity } => {
                    println!("Detached from broken identity: {identity}");
                }
                AuthStatus::Native if force => {
                    println!("Discarded native auth file");
                }
                AuthStatus::None | AuthStatus::CodexHomeMissing { .. } => {
                    println!("No active identity");
                }
                AuthStatus::Native | AuthStatus::Unknown { .. } => {}
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    Status,
    List,
    Capture { identity: IdentityName, force: bool },
    Use { identity: IdentityName, force: bool },
    Detach { force: bool },
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    match args {
        [] => Ok(Command::Status),
        [flag] if flag == "--help" || flag == "-h" || flag == "help" => Ok(Command::Help),
        [flag] if flag == "--version" || flag == "-V" => Ok(Command::Version),
        [command] if command == "status" => Ok(Command::Status),
        [command] if command == "list" => Ok(Command::List),
        [command, rest @ ..] if command == "capture" => {
            parse_identity_command(rest, CommandKind::Capture)
        }
        [command, rest @ ..] if command == "use" => parse_identity_command(rest, CommandKind::Use),
        [command, rest @ ..] if command == "detach" => parse_detach_command(rest),
        [command, ..] => Err(CliError::Usage(format!("unknown command: {command}"))),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Capture,
    Use,
}

fn parse_identity_command(args: &[String], kind: CommandKind) -> Result<Command, CliError> {
    let mut identity = None;
    let mut force = false;
    for arg in args {
        if arg == "--force" {
            if force {
                return Err(CliError::Usage("duplicate flag: --force".to_owned()));
            }
            force = true;
        } else if arg.starts_with('-') {
            return Err(CliError::Usage(format!("unknown flag: {arg}")));
        } else if identity.is_some() {
            return Err(CliError::Usage(format!("unexpected argument: {arg}")));
        } else {
            identity = Some(IdentityName::from_str(arg).map_err(identity_parse_error)?);
        }
    }
    let identity = identity.ok_or_else(|| {
        CliError::Usage(match kind {
            CommandKind::Capture => "missing identity for capture".to_owned(),
            CommandKind::Use => "missing identity for use".to_owned(),
        })
    })?;
    Ok(match kind {
        CommandKind::Capture => Command::Capture { identity, force },
        CommandKind::Use => Command::Use { identity, force },
    })
}

fn parse_detach_command(args: &[String]) -> Result<Command, CliError> {
    let mut force = false;
    for arg in args {
        if arg == "--force" {
            if force {
                return Err(CliError::Usage("duplicate flag: --force".to_owned()));
            }
            force = true;
        } else if arg.starts_with('-') {
            return Err(CliError::Usage(format!("unknown flag: {arg}")));
        } else {
            return Err(CliError::Usage(format!("unexpected argument: {arg}")));
        }
    }
    Ok(Command::Detach { force })
}

fn print_help() {
    println!(
        "\
cam manages named Codex auth identities.

Usage:
  cam
  cam status
  cam list
  cam capture <identity> [--force]
  cam use <identity> [--force]
  cam detach [--force]
  cam help

Identity names must match: [a-zA-Z0-9][a-zA-Z0-9._-]*
"
    );
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Runtime(Error),
}

impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        Self::Runtime(error)
    }
}

fn identity_parse_error(error: Error) -> CliError {
    match error {
        Error::InvalidIdentityName { name } => {
            CliError::Usage(format!("invalid identity name: {name}"))
        }
        other => CliError::Runtime(other),
    }
}
