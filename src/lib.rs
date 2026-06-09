//! # `codex-auth-manager` library crate
//!
//! If you are reading this, you are reading the documentation for the `codex-auth-manager` library crate. For the cli, kindly refer to the README file.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use std::{env::var, path::PathBuf};

/// Determine the path to Codex's home directory. ($CODEX_HOME or $HOME/.codex)
pub fn codex_home() -> Option<PathBuf> {
    var("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|_| var("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .ok()
}
