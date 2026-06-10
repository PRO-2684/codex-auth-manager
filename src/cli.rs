use std::ffi::OsStr;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{ArgValueCompleter, CompleteEnv, CompletionCandidate};
use codex_auth_manager::{
    AuthStatus, CaptureOptions, CodexAuthManager, DetachOptions, Error, IdentityName,
    PKG_DESCRIPTION, UseOptions,
};

pub fn complete_from_env() {
    CompleteEnv::with_factory(Cli::command).complete();
}

pub fn run() -> Result<(), CliError> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) if error.use_stderr() => return Err(CliError::Usage(error)),
        Err(error) => {
            let _ = error.print();
            return Ok(());
        }
    };

    match cli.command.unwrap_or(Command::Status) {
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

#[derive(Debug, Parser)]
#[command(
    name = "cam",
    version,
    about = PKG_DESCRIPTION,
    after_help = "Identity names must match: [a-zA-Z0-9][a-zA-Z0-9._-]*"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show the current auth state.
    Status,
    /// List saved identities.
    List,
    /// Save the current native Codex auth file as an identity and make it active.
    Capture {
        /// Identity to create or overwrite.
        #[arg(add = ArgValueCompleter::new(identity_completer))]
        identity: IdentityName,
        /// Overwrite an existing regular identity file.
        #[arg(long)]
        force: bool,
    },
    /// Make an existing identity active.
    Use {
        /// Identity to activate.
        #[arg(add = ArgValueCompleter::new(identity_completer))]
        identity: IdentityName,
        /// Discard a blocking native auth file.
        #[arg(long)]
        force: bool,
    },
    /// Stop using the active CAM-managed identity.
    Detach {
        /// Remove a blocking native auth file or broken managed link.
        #[arg(long)]
        force: bool,
    },
}

fn identity_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };
    let Ok(manager) = CodexAuthManager::from_env() else {
        return Vec::new();
    };
    let Ok(identities) = manager.list() else {
        return Vec::new();
    };

    identities
        .into_iter()
        .filter(|identity| !identity.broken)
        .filter(|identity| identity.name.as_str().starts_with(current))
        .map(|identity| {
            let candidate = CompletionCandidate::new(identity.name.as_str().to_owned());
            if identity.active {
                candidate.help(Some("active".into()))
            } else {
                candidate
            }
        })
        .collect()
}

#[derive(Debug)]
pub enum CliError {
    Usage(clap::Error),
    Runtime(Error),
}

impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        Self::Runtime(error)
    }
}
