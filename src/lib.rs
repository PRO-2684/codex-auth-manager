//! Library API for managing named Codex auth identities.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

mod error;
mod fs;
mod identity;
mod manager;
mod status;

pub use error::Error;
pub use identity::{Identity, IdentityName};
pub use manager::{CaptureOptions, CodexAuthManager, DetachOptions, UseOptions};
pub use status::{AuthStatus, UnknownAuthReason};

/// Package name.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
