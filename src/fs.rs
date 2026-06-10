use std::{
    env::current_dir,
    fs,
    path::{Component, Path, PathBuf},
};

use super::Error;

pub const AUTH_FILE: &str = "auth.json";
pub const STORAGE_SUFFIX: &str = ".json";

pub fn absolutize(path: &Path) -> Result<PathBuf, Error> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(current_dir()
            .map_err(|source| Error::CurrentDir { source })?
            .join(path))
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub fn is_broken_identity_path(path: &Path) -> Result<bool, Error> {
    fs::symlink_metadata(path)
        .map(|metadata| !metadata.file_type().is_file())
        .map_err(|source| Error::Io {
            action: "inspect identity",
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(unix)]
pub fn create_symlink(target: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub fn create_symlink(target: impl AsRef<Path>, link: impl AsRef<Path>) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

pub fn replace_file(from: &Path, to: &Path) -> std::io::Result<()> {
    replace_file_impl(from, to)
}

#[cfg(unix)]
fn replace_file_impl(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::rename(from, to)
}

#[cfg(windows)]
fn replace_file_impl(from: &Path, to: &Path) -> std::io::Result<()> {
    if to.exists() || to.is_symlink() {
        fs::remove_file(to)?;
    }
    fs::rename(from, to)
}
