use std::{fmt, path::PathBuf, str::FromStr};

use super::Error;

#[cfg(feature = "identity-details")]
mod details;

#[cfg(feature = "identity-details")]
pub use details::IdentityDetails;

/// A valid identity name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityName(String);

impl IdentityName {
    /// Borrow the identity name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for IdentityName {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_identity_name(value)?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for IdentityName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_identity_name(&value)?;
        Ok(Self(value))
    }
}

impl FromStr for IdentityName {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl AsRef<str> for IdentityName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for IdentityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An identity entry in the manager directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Identity name.
    pub name: IdentityName,
    /// Storage path.
    pub path: PathBuf,
    /// Whether this identity is selected by `auth.json`.
    pub active: bool,
    /// Whether this identity entry exists but is unusable.
    pub broken: bool,
}

fn validate_identity_name(value: &str) -> Result<(), Error> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(Error::InvalidIdentityName {
            name: value.to_owned(),
        });
    };
    if !first.is_ascii_alphanumeric()
        || !chars.all(|char| char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-'))
    {
        return Err(Error::InvalidIdentityName {
            name: value.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Error, IdentityName};

    #[test]
    fn identity_name_validation_accepts_portable_names() {
        for name in ["personal", "OpenAI-Work", "org.dev", "test_2", "work.json"] {
            assert!(IdentityName::try_from(name).is_ok());
        }
    }

    #[test]
    fn identity_name_validation_rejects_paths_and_shellish_names() {
        for name in ["", "-prod", "my work", "../auth", "work/main", "work\\main"] {
            assert!(matches!(
                IdentityName::try_from(name),
                Err(Error::InvalidIdentityName { .. })
            ));
        }
    }
}
