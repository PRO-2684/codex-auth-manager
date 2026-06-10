use std::path::PathBuf;

use thiserror::Error;

use super::{IdentityName, UnknownAuthReason};

/// Error returned by the library API.
#[derive(Debug, Error)]
pub enum Error {
    /// Environment did not contain enough information to resolve Codex home.
    #[error("failed to determine Codex home: {source}")]
    Env {
        /// Source environment error.
        source: std::env::VarError,
    },
    /// Current working directory could not be read while absolutizing a path.
    #[error("failed to determine current directory: {source}")]
    CurrentDir {
        /// Source filesystem error.
        source: std::io::Error,
    },
    /// Filesystem operation failed.
    #[error("failed to {action} at {}: {source}", path.display())]
    Io {
        /// Action being attempted.
        action: &'static str,
        /// Path involved in the failure.
        path: PathBuf,
        /// Source filesystem error.
        source: std::io::Error,
    },
    /// Identity name is invalid.
    #[error("invalid identity name: {name}")]
    InvalidIdentityName {
        /// Invalid name.
        name: String,
    },
    /// Identity does not exist.
    #[error("identity not found: {name}")]
    IdentityNotFound {
        /// Missing identity name.
        name: IdentityName,
    },
    /// Identity already exists.
    #[error("identity already exists: {name}")]
    IdentityAlreadyExists {
        /// Existing identity name.
        name: IdentityName,
    },
    /// Identity entry exists but is unusable.
    #[error("identity is broken: {name}")]
    IdentityBroken {
        /// Broken identity name.
        name: IdentityName,
    },
    /// Native auth file exists and would be discarded.
    #[error("native auth file exists; capture it first or pass --force to discard it")]
    NativeAuthExists,
    /// No native auth file exists to capture.
    #[error("no native auth file to capture")]
    NoNativeAuthFile,
    /// Codex home does not exist.
    #[error("codex home missing: {}", path.display())]
    CodexHomeMissing {
        /// Missing Codex home path.
        path: PathBuf,
    },
    /// Current auth state is unknown.
    #[error("unknown auth state: {reason}")]
    UnknownAuthState {
        /// Reason the auth state is unknown.
        reason: UnknownAuthReason,
    },
}
