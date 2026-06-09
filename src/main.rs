#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use codex_auth_manager::{AuthStatus, codex_home};

fn main() {
    let Some(codex_home) = codex_home() else {
        eprintln!("Failed to determine Codex home directory.");
        return;
    };
    eprintln!("Codex home directory: {}", codex_home.display());
    let status = AuthStatus::determine_from(&codex_home);
    println!("{status}");
}
