use std::{fmt, path::PathBuf};

use super::IdentitySlug;

/// Current auth status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthStatus {
    /// Codex home does not exist.
    CodexHomeMissing {
        /// Missing Codex home path.
        path: PathBuf,
    },
    /// No auth file exists.
    None,
    /// `auth.json` is a regular native Codex auth file.
    Native,
    /// `auth.json` points to a usable managed identity.
    Managed {
        /// Active identity slug.
        slug: IdentitySlug,
    },
    /// `auth.json` points to a managed identity that is missing or unusable.
    BrokenManaged {
        /// Broken active identity slug.
        slug: IdentitySlug,
    },
    /// CAM cannot safely classify the auth state.
    Unknown {
        /// Reason the state is unknown.
        reason: UnknownAuthReason,
    },
}

impl fmt::Display for AuthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodexHomeMissing { path } => write!(f, "Codex home missing: {}", path.display()),
            Self::None => write!(f, "No auth file"),
            Self::Native => write!(f, "Native auth file"),
            Self::Managed { slug } => write!(f, "Active identity: {slug}"),
            Self::BrokenManaged { slug } => {
                write!(f, "Active identity is broken: {slug}")
            }
            Self::Unknown { reason } => write!(f, "Unknown auth state: {reason}"),
        }
    }
}

/// Reason CAM cannot safely classify the auth state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownAuthReason {
    /// `auth.json` is neither a regular file nor a symlink.
    AuthPathIsNotFileOrSymlink,
    /// The `auth.json` symlink points outside CAM's manager directory.
    SymlinkTargetOutsideManagerDir,
    /// The `auth.json` symlink target does not map to a valid identity slug.
    SymlinkTargetHasInvalidIdentitySlug,
}

impl fmt::Display for UnknownAuthReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthPathIsNotFileOrSymlink => write!(f, "auth.json is not a file or symlink"),
            Self::SymlinkTargetOutsideManagerDir => {
                write!(f, "symlink target is outside codex-auth-manager")
            }
            Self::SymlinkTargetHasInvalidIdentitySlug => {
                write!(f, "symlink target has invalid identity slug")
            }
        }
    }
}
