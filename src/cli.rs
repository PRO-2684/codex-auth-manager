use std::ffi::OsStr;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{ArgValueCompleter, CompleteEnv, CompletionCandidate};
use codex_auth_manager::{
    AuthStatus, CaptureOptions, CodexAuthManager, DetachOptions, Error, Identity, IdentityDetails,
    IdentitySlug, PKG_DESCRIPTION, UseOptions,
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

    match cli.command.unwrap_or_default() {
        Command::Status => {
            let manager = CodexAuthManager::from_env()?;
            let status = manager.status()?;
            let details = manager.read_active_auth_details().ok().flatten();
            println!("{}", format_status(&status, details.as_ref()));
            Ok(())
        }
        Command::List => {
            let manager = CodexAuthManager::from_env()?;
            for identity in manager.list()? {
                let details = (!identity.broken)
                    .then(|| identity.read_details().ok().flatten())
                    .flatten();
                println!("{}", format_identity(&identity, details.as_ref()));
            }
            Ok(())
        }
        Command::Capture { slug, force } => {
            let manager = CodexAuthManager::from_env()?;
            manager.capture(&slug, CaptureOptions { force })?;
            println!("Captured identity: {slug}");
            Ok(())
        }
        Command::Use { slug, force } => {
            let manager = CodexAuthManager::from_env()?;
            manager.use_identity(&slug, UseOptions { force })?;
            println!("Active identity: {slug}");
            Ok(())
        }
        Command::Detach { force } => {
            let manager = CodexAuthManager::from_env()?;
            let status = manager.status()?;
            manager.detach(DetachOptions { force })?;
            match status {
                AuthStatus::Managed { slug } => {
                    println!("Detached from active identity: {slug}");
                }
                AuthStatus::BrokenManaged { slug } => {
                    println!("Detached from broken identity: {slug}");
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

fn format_identity(identity: &Identity, details: Option<&IdentityDetails>) -> String {
    let marker = if identity.active { "*" } else { " " };
    let broken = if identity.broken { " (broken)" } else { "" };
    let details = details.map_or_else(String::new, |details| format!(" — {details}"));
    format!("{marker} {}{broken}{details}", identity.slug)
}

fn format_status(status: &AuthStatus, details: Option<&IdentityDetails>) -> String {
    let details = details.map_or_else(String::new, |details| format!(" — {details}"));
    format!("{status}{details}")
}

#[derive(Debug, Parser)]
#[command(name = "cam", version, about = PKG_DESCRIPTION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Default, Subcommand)]
enum Command {
    /// Show the current auth state.
    #[default]
    Status,
    /// List saved identities.
    List,
    /// Save the current native Codex auth file as an identity and make it active.
    Capture {
        /// Identity slug to create or overwrite.
        #[arg(add = ArgValueCompleter::new(identity_completer))]
        slug: IdentitySlug,
        /// Overwrite an existing regular identity file.
        #[arg(long)]
        force: bool,
    },
    /// Make an existing identity active.
    Use {
        /// Identity slug to activate.
        #[arg(add = ArgValueCompleter::new(identity_completer))]
        slug: IdentitySlug,
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
        .filter(|identity| identity.slug.as_str().starts_with(current))
        .map(|identity| {
            let candidate = CompletionCandidate::new(identity.slug.as_str().to_owned());
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use codex_auth_manager::{AuthStatus, Identity, IdentityDetails, IdentitySlug};

    use super::{format_identity, format_status};

    #[test]
    fn list_line_includes_identity_details() {
        let identity = Identity {
            slug: IdentitySlug::try_from("personal").unwrap(),
            path: PathBuf::from("personal.json"),
            active: true,
            broken: false,
        };
        let details = IdentityDetails {
            display_name: Some("Example User".to_owned()),
            email: Some("the.user@gmail.com".to_owned()),
        };

        assert_eq!(
            format_identity(&identity, Some(&details)),
            "* personal — Example User <the.user@gmail.com>"
        );
    }

    #[test]
    fn status_line_includes_active_auth_details() {
        let details = IdentityDetails {
            display_name: Some("Example User".to_owned()),
            email: Some("the.user@gmail.com".to_owned()),
        };
        let status = AuthStatus::Managed {
            slug: IdentitySlug::try_from("personal").unwrap(),
        };

        assert_eq!(
            format_status(&status, Some(&details)),
            "Active identity: personal — Example User <the.user@gmail.com>"
        );
    }
}
