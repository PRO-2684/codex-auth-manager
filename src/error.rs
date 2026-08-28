use std::{env::VarError, io::Error as IoError, path::PathBuf};

use declerror::error_enum;
use macro_rules_attr::apply;

use super::{IdentitySlug, UnknownAuthReason};

/// Error returned by the library API.
#[apply(error_enum)]
pub enum Error {
    /// Environment did not contain enough information to resolve Codex home.
    #[error("failed to determine Codex home: {source}")]
    Env {
        /// Source environment error.
        source: VarError,
    },
    /// Current working directory could not be read while absolutizing a path.
    #[error("failed to determine current directory: {source}")]
    CurrentDir {
        /// Source filesystem error.
        source: IoError,
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
    /// Identity slug is invalid.
    #[error("invalid identity slug: {slug}")]
    InvalidIdentitySlug {
        /// Invalid slug.
        slug: String,
    },
    /// Identity does not exist.
    #[error("identity not found: {slug}")]
    IdentityNotFound {
        /// Missing identity slug.
        slug: IdentitySlug,
    },
    /// Identity already exists.
    #[error("identity already exists: {slug}")]
    IdentityAlreadyExists {
        /// Existing identity slug.
        slug: IdentitySlug,
    },
    /// Identity entry exists but is unusable.
    #[error("identity is broken: {slug}")]
    IdentityBroken {
        /// Broken identity slug.
        slug: IdentitySlug,
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
