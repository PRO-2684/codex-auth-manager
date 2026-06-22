use std::{
    env::var,
    fs,
    path::{Path, PathBuf},
};

use super::{
    AuthStatus, Error, Identity, IdentityName, PKG_NAME, UnknownAuthReason,
    fs::{
        AUTH_FILE, STORAGE_SUFFIX, absolutize, create_symlink, is_broken_identity_path,
        normalize_path, replace_file,
    },
};

/// A manager rooted at one Codex home directory.
#[derive(Debug, Clone)]
pub struct CodexAuthManager {
    codex_home: PathBuf,
}

impl CodexAuthManager {
    /// Create a manager from `$CODEX_HOME`, or `$HOME/.codex` when `$CODEX_HOME` is unset.
    ///
    /// The stored path is absolute. The directory is not created by this method.
    ///
    /// # Errors
    ///
    /// Returns an error when neither environment variable can be read or the current directory
    /// cannot be determined while absolutizing a relative path.
    pub fn from_env() -> Result<Self, Error> {
        let path = var("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|_| var("HOME").map(|home| PathBuf::from(home).join(".codex")))
            .map_err(|source| Error::Env { source })?;
        Self::new(path)
    }

    /// Create a manager for `codex_home`.
    ///
    /// The stored path is absolute. The directory is not created by this method.
    ///
    /// # Errors
    ///
    /// Returns an error when the current directory cannot be determined while absolutizing a
    /// relative path.
    pub fn new(codex_home: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            codex_home: absolutize(codex_home.as_ref())?,
        })
    }

    /// The Codex home directory this manager uses.
    #[must_use]
    pub fn codex_home(&self) -> &Path {
        &self.codex_home
    }

    /// Return the current auth status.
    ///
    /// # Errors
    ///
    /// Returns an error when CAM cannot inspect `auth.json` or read its symlink target.
    pub fn status(&self) -> Result<AuthStatus, Error> {
        if !self.codex_home.exists() {
            return Ok(AuthStatus::CodexHomeMissing {
                path: self.codex_home.clone(),
            });
        }

        let auth_path = self.auth_path();
        let metadata = match fs::symlink_metadata(&auth_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(AuthStatus::None);
            }
            Err(source) => {
                return Err(Error::Io {
                    action: "inspect auth file",
                    path: auth_path,
                    source,
                });
            }
        };

        let file_type = metadata.file_type();
        if file_type.is_file() {
            return Ok(AuthStatus::Native);
        }
        if file_type.is_symlink() {
            return self.status_from_symlink(&auth_path);
        }

        Ok(AuthStatus::Unknown {
            reason: UnknownAuthReason::AuthPathIsNotFileOrSymlink,
        })
    }

    /// List identities from the manager directory.
    ///
    /// Broken identity entries are included. Missing manager directories produce an empty list.
    ///
    /// # Errors
    ///
    /// Returns an error when the manager directory or one of its entries cannot be inspected.
    pub fn list(&self) -> Result<Vec<Identity>, Error> {
        let status = self.status()?;
        let active = match &status {
            AuthStatus::Managed { identity } | AuthStatus::BrokenManaged { identity } => Some((
                identity.clone(),
                matches!(status, AuthStatus::BrokenManaged { .. }),
            )),
            AuthStatus::None
            | AuthStatus::Native
            | AuthStatus::CodexHomeMissing { .. }
            | AuthStatus::Unknown { .. } => None,
        };

        let mut identities = Vec::new();
        let manager_dir = self.manager_dir();
        let entries = match fs::read_dir(&manager_dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if let Some((identity, broken)) = active {
                    identities.push(Identity {
                        path: self.identity_path(&identity),
                        name: identity,
                        active: true,
                        broken,
                    });
                }
                return Ok(identities);
            }
            Err(source) => {
                return Err(Error::Io {
                    action: "read identity directory",
                    path: manager_dir,
                    source,
                });
            }
        };

        for entry in entries {
            let entry = entry.map_err(|source| Error::Io {
                action: "read identity directory entry",
                path: manager_dir.clone(),
                source,
            })?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Some(name) = file_name.strip_suffix(STORAGE_SUFFIX) else {
                continue;
            };
            let Ok(name) = IdentityName::try_from(name) else {
                continue;
            };
            let path = entry.path();
            let broken = is_broken_identity_path(&path)?;
            let active_here = active
                .as_ref()
                .is_some_and(|(active_name, _)| active_name == &name);
            identities.push(Identity {
                path,
                name,
                active: active_here,
                broken,
            });
        }

        if let Some((active_name, active_broken)) = active
            && !identities
                .iter()
                .any(|identity| identity.name == active_name)
        {
            identities.push(Identity {
                path: self.identity_path(&active_name),
                name: active_name,
                active: true,
                broken: active_broken,
            });
        }

        identities.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(identities)
    }

    /// Capture the native auth file into an identity and make it active.
    ///
    /// # Errors
    ///
    /// Returns an error when there is no native auth file, the destination identity already
    /// exists without `force`, the destination is broken, or a filesystem operation fails.
    pub fn capture(&self, identity: &IdentityName, options: CaptureOptions) -> Result<(), Error> {
        self.require_codex_home()?;
        match self.status()? {
            AuthStatus::Native => {}
            AuthStatus::None
            | AuthStatus::CodexHomeMissing { .. }
            | AuthStatus::Managed { .. }
            | AuthStatus::BrokenManaged { .. } => {
                return Err(Error::NoNativeAuthFile);
            }
            AuthStatus::Unknown { reason } => return Err(Error::UnknownAuthState { reason }),
        }

        let manager_dir = self.manager_dir();
        fs::create_dir_all(&manager_dir).map_err(|source| Error::Io {
            action: "create identity directory",
            path: manager_dir,
            source,
        })?;

        let identity_path = self.identity_path(identity);
        match fs::symlink_metadata(&identity_path) {
            Ok(metadata) if metadata.file_type().is_file() && options.force => {}
            Ok(metadata) if metadata.file_type().is_file() => {
                return Err(Error::IdentityAlreadyExists {
                    name: identity.clone(),
                });
            }
            Ok(_) => {
                return Err(Error::IdentityBroken {
                    name: identity.clone(),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(Error::Io {
                    action: "inspect identity",
                    path: identity_path,
                    source,
                });
            }
        }

        let auth_path = self.auth_path();
        let tmp_path = self.create_temporary_auth_symlink(identity)?;
        replace_file(&auth_path, &identity_path).map_err(|source| Error::Io {
            action: "capture native auth file",
            path: auth_path.clone(),
            source,
        })?;
        replace_file(&tmp_path, &auth_path)
            .map_err(|source| Error::Io {
                action: "activate identity",
                path: auth_path.clone(),
                source,
            })
            .inspect_err(|_| {
                let _ = fs::rename(&identity_path, &auth_path);
                let _ = fs::remove_file(&tmp_path);
            })
    }

    /// Make an existing identity active.
    ///
    /// # Errors
    ///
    /// Returns an error when the identity is missing or broken, a native auth file would be
    /// discarded without `force`, the auth state is unknown, or a filesystem operation fails.
    pub fn use_identity(&self, identity: &IdentityName, options: UseOptions) -> Result<(), Error> {
        self.require_codex_home()?;
        self.require_usable_identity(identity)?;

        match self.status()? {
            AuthStatus::None | AuthStatus::Managed { .. } | AuthStatus::BrokenManaged { .. } => {}
            AuthStatus::Native if options.force => {
                fs::remove_file(self.auth_path()).map_err(|source| Error::Io {
                    action: "discard native auth file",
                    path: self.auth_path(),
                    source,
                })?;
            }
            AuthStatus::Native => return Err(Error::NativeAuthExists),
            AuthStatus::CodexHomeMissing { path } => return Err(Error::CodexHomeMissing { path }),
            AuthStatus::Unknown { reason } => return Err(Error::UnknownAuthState { reason }),
        }

        self.replace_auth_symlink(identity)
    }

    /// Detach Codex from the active managed identity.
    ///
    /// # Errors
    ///
    /// Returns an error when detaching would discard a native auth file or broken managed link
    /// without `force`, the auth state is unknown, or a filesystem operation fails.
    pub fn detach(&self, options: DetachOptions) -> Result<(), Error> {
        match self.status()? {
            AuthStatus::None | AuthStatus::CodexHomeMissing { .. } => Ok(()),
            AuthStatus::Managed { .. } => self.remove_auth_file(),
            AuthStatus::BrokenManaged { .. } | AuthStatus::Native if options.force => {
                self.remove_auth_file()
            }
            AuthStatus::BrokenManaged { identity } => Err(Error::IdentityBroken { name: identity }),
            AuthStatus::Native => Err(Error::NativeAuthExists),
            AuthStatus::Unknown { reason } => Err(Error::UnknownAuthState { reason }),
        }
    }

    fn auth_path(&self) -> PathBuf {
        self.codex_home.join(AUTH_FILE)
    }

    fn manager_dir(&self) -> PathBuf {
        self.codex_home.join(PKG_NAME)
    }

    fn identity_path(&self, identity: &IdentityName) -> PathBuf {
        self.manager_dir()
            .join(format!("{identity}{STORAGE_SUFFIX}"))
    }

    fn relative_identity_path(identity: &IdentityName) -> PathBuf {
        PathBuf::from(PKG_NAME).join(format!("{identity}{STORAGE_SUFFIX}"))
    }

    fn require_codex_home(&self) -> Result<(), Error> {
        if self.codex_home.exists() {
            Ok(())
        } else {
            Err(Error::CodexHomeMissing {
                path: self.codex_home.clone(),
            })
        }
    }

    fn require_usable_identity(&self, identity: &IdentityName) -> Result<(), Error> {
        let path = self.identity_path(identity);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(()),
            Ok(_) => Err(Error::IdentityBroken {
                name: identity.clone(),
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(Error::IdentityNotFound {
                    name: identity.clone(),
                })
            }
            Err(source) => Err(Error::Io {
                action: "inspect identity",
                path,
                source,
            }),
        }
    }

    fn status_from_symlink(&self, auth_path: &Path) -> Result<AuthStatus, Error> {
        let target = fs::read_link(auth_path).map_err(|source| Error::Io {
            action: "read auth symlink",
            path: auth_path.to_path_buf(),
            source,
        })?;
        let target = if target.is_absolute() {
            target
        } else {
            self.codex_home.join(target)
        };
        let target = normalize_path(&target);
        let manager_dir = normalize_path(&self.manager_dir());
        let Ok(relative) = target.strip_prefix(&manager_dir) else {
            return Ok(AuthStatus::Unknown {
                reason: UnknownAuthReason::SymlinkTargetOutsideManagerDir,
            });
        };
        if relative.components().count() != 1 {
            return Ok(AuthStatus::Unknown {
                reason: UnknownAuthReason::SymlinkTargetOutsideManagerDir,
            });
        }
        let Some(file_name) = relative.file_name().and_then(|name| name.to_str()) else {
            return Ok(AuthStatus::Unknown {
                reason: UnknownAuthReason::SymlinkTargetHasInvalidIdentityName,
            });
        };
        let Some(name) = file_name.strip_suffix(STORAGE_SUFFIX) else {
            return Ok(AuthStatus::Unknown {
                reason: UnknownAuthReason::SymlinkTargetHasInvalidIdentityName,
            });
        };
        let identity = IdentityName::try_from(name).map_err(|_| Error::UnknownAuthState {
            reason: UnknownAuthReason::SymlinkTargetHasInvalidIdentityName,
        })?;
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(AuthStatus::Managed { identity }),
            Ok(_) | Err(_) => Ok(AuthStatus::BrokenManaged { identity }),
        }
    }

    fn replace_auth_symlink(&self, identity: &IdentityName) -> Result<(), Error> {
        let tmp_path = self.create_temporary_auth_symlink(identity)?;
        replace_file(&tmp_path, &self.auth_path()).map_err(|source| Error::Io {
            action: "activate identity",
            path: self.auth_path(),
            source,
        })
    }

    fn create_temporary_auth_symlink(&self, identity: &IdentityName) -> Result<PathBuf, Error> {
        let tmp_path = self.codex_home.join(format!(".{AUTH_FILE}.tmp"));
        if tmp_path.exists() || tmp_path.is_symlink() {
            fs::remove_file(&tmp_path).map_err(|source| Error::Io {
                action: "remove stale temporary auth symlink",
                path: tmp_path.clone(),
                source,
            })?;
        }
        create_symlink(Self::relative_identity_path(identity), &tmp_path).map_err(|source| {
            Error::Io {
                action: "create temporary auth symlink",
                path: tmp_path.clone(),
                source,
            }
        })?;
        Ok(tmp_path)
    }

    fn remove_auth_file(&self) -> Result<(), Error> {
        fs::remove_file(self.auth_path()).map_err(|source| Error::Io {
            action: "remove auth file",
            path: self.auth_path(),
            source,
        })
    }
}

/// Options for capturing a native auth file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaptureOptions {
    /// Overwrite an existing regular identity file.
    pub force: bool,
}

/// Options for using an identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UseOptions {
    /// Discard a blocking native auth file.
    pub force: bool,
}

/// Options for detaching from the active identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DetachOptions {
    /// Remove a blocking native auth file or broken managed link.
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        AuthStatus, CaptureOptions, CodexAuthManager, DetachOptions, Error, IdentityName,
        UseOptions, fs::create_symlink,
    };

    #[test]
    fn capture_moves_native_auth_and_marks_identity_active() {
        let temp = TempHome::new();
        fs::create_dir_all(temp.path()).unwrap();
        fs::write(temp.path().join("auth.json"), "{}").unwrap();
        let manager = CodexAuthManager::new(temp.path()).unwrap();
        let identity = IdentityName::try_from("work").unwrap();

        manager
            .capture(&identity, CaptureOptions::default())
            .unwrap();

        assert_eq!(
            manager.status().unwrap(),
            AuthStatus::Managed {
                identity: identity.clone()
            }
        );
        assert_eq!(
            fs::read_to_string(temp.path().join("codex-auth-manager/work.json")).unwrap(),
            "{}"
        );
        assert_eq!(
            fs::read_link(temp.path().join("auth.json")).unwrap(),
            PathBuf::from("codex-auth-manager/work.json")
        );
        let identities = manager.list().unwrap();
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].name, identity);
        assert!(identities[0].active);
        assert!(!identities[0].broken);
    }

    #[test]
    fn capture_force_overwrites_existing_regular_identity() {
        let temp = TempHome::new();
        fs::create_dir_all(temp.path().join("codex-auth-manager")).unwrap();
        fs::write(temp.path().join("auth.json"), "new").unwrap();
        fs::write(temp.path().join("codex-auth-manager/work.json"), "old").unwrap();
        let manager = CodexAuthManager::new(temp.path()).unwrap();
        let identity = IdentityName::try_from("work").unwrap();

        assert!(matches!(
            manager.capture(&identity, CaptureOptions::default()),
            Err(Error::IdentityAlreadyExists { .. })
        ));
        manager
            .capture(&identity, CaptureOptions { force: true })
            .unwrap();

        assert_eq!(
            fs::read_to_string(temp.path().join("codex-auth-manager/work.json")).unwrap(),
            "new"
        );
    }

    #[test]
    fn use_identity_refuses_native_auth_without_force() {
        let temp = TempHome::new();
        fs::create_dir_all(temp.path().join("codex-auth-manager")).unwrap();
        fs::write(temp.path().join("auth.json"), "native").unwrap();
        fs::write(temp.path().join("codex-auth-manager/work.json"), "work").unwrap();
        let manager = CodexAuthManager::new(temp.path()).unwrap();
        let identity = IdentityName::try_from("work").unwrap();

        assert!(matches!(
            manager.use_identity(&identity, UseOptions::default()),
            Err(Error::NativeAuthExists)
        ));
        manager
            .use_identity(&identity, UseOptions { force: true })
            .unwrap();

        assert_eq!(manager.status().unwrap(), AuthStatus::Managed { identity });
    }

    #[test]
    fn list_includes_broken_active_identity() {
        let temp = TempHome::new();
        fs::create_dir_all(temp.path()).unwrap();
        create_symlink(
            PathBuf::from("codex-auth-manager/work.json"),
            temp.path().join("auth.json"),
        )
        .unwrap();
        let manager = CodexAuthManager::new(temp.path()).unwrap();

        assert_eq!(
            manager.status().unwrap(),
            AuthStatus::BrokenManaged {
                identity: IdentityName::try_from("work").unwrap()
            }
        );
        let identities = manager.list().unwrap();
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].name.as_str(), "work");
        assert!(identities[0].active);
        assert!(identities[0].broken);
    }

    #[test]
    fn detach_force_removes_broken_managed_link() {
        let temp = TempHome::new();
        fs::create_dir_all(temp.path()).unwrap();
        create_symlink(
            PathBuf::from("codex-auth-manager/work.json"),
            temp.path().join("auth.json"),
        )
        .unwrap();
        let manager = CodexAuthManager::new(temp.path()).unwrap();

        assert!(matches!(
            manager.detach(DetachOptions::default()),
            Err(Error::IdentityBroken { .. })
        ));
        manager.detach(DetachOptions { force: true }).unwrap();
        assert_eq!(manager.status().unwrap(), AuthStatus::None);
    }

    struct TempHome {
        path: PathBuf,
    }

    impl TempHome {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self {
                path: std::env::temp_dir().join(format!(
                    "codex-auth-manager-test-{}-{unique}",
                    std::process::id()
                )),
            }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
