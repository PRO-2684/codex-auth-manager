//! # `codex-auth-manager` library crate
//!
//! If you are reading this, you are reading the documentation for the `codex-auth-manager` library crate. For the cli, kindly refer to the README file.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use std::{
    env::var,
    fmt,
    path::{Path, PathBuf},
};

/// Package name.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Determine the path to Codex's home directory. ($CODEX_HOME or $HOME/.codex)
pub fn codex_home() -> Option<PathBuf> {
    var("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|_| var("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .ok()
}

/// Current auth status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthStatus {
    /// Not authenticated. (auth.json file is missing)
    None,
    /// Authenticated by native Codex. (auth.json file exists and is a normal file)
    Native,
    /// Authentication managed by us. (auth.json file is a symlink to our auth directory $CODEX_HOME/$PKG_NAME)
    Managed,
    /// Cannot determine auth status. (auth.json file exists but is neither a normal file nor a symlink to our auth directory)
    Unknown,
}

impl AuthStatus {
    /// Determine the auth status.
    pub fn determine() -> Self {
        codex_home()
            .map(Self::determine_from)
            .unwrap_or(Self::Unknown)
    }

    /// Determine the auth status at given Codex home by checking the existence and type of the `auth.json` file.
    pub fn determine_from<P: AsRef<Path>>(codex_home: P) -> Self {
        let codex_home = codex_home.as_ref();
        let auth_path = codex_home.join("auth.json");
        if !auth_path.exists() {
            Self::None
        } else if auth_path.is_file() {
            Self::Native
        } else if auth_path.is_symlink()
            && auth_path
                .read_link()
                .map_or(false, |target| target == codex_home.join(PKG_NAME))
        {
            Self::Managed
        } else {
            Self::Unknown
        }
    }
}

impl fmt::Display for AuthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Not authenticated"),
            Self::Native => write!(f, "Authenticated by native Codex"),
            Self::Managed => write!(f, "Authentication managed by us"),
            Self::Unknown => write!(f, "Unknown authentication status"),
        }
    }
}
