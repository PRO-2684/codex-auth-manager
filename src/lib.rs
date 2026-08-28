//! Library API for managing named Codex auth identities.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod error;
mod fs;
mod identity;
mod manager;
mod status;

pub use error::Error;
#[cfg(feature = "identity-details")]
pub use identity::IdentityDetails;
pub use identity::{Identity, IdentitySlug};
pub use manager::{CaptureOptions, CodexAuthManager, DetachOptions, UseOptions};
pub use status::{AuthStatus, UnknownAuthReason};

/// Package name.
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Package description.
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
