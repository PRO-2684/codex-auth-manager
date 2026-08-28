use std::{fmt, path::PathBuf, str::FromStr};

use super::Error;

#[cfg(feature = "identity-details")]
mod details;

#[cfg(feature = "identity-details")]
pub use details::IdentityDetails;

/// A valid identity slug.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentitySlug(String);

impl IdentitySlug {
    /// Borrow the identity slug as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for IdentitySlug {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_identity_slug(value)?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for IdentitySlug {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        validate_identity_slug(&value)?;
        Ok(Self(value))
    }
}

impl FromStr for IdentitySlug {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl AsRef<str> for IdentitySlug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for IdentitySlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An identity entry in the manager directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Portable user-defined identity identifier.
    pub slug: IdentitySlug,
    /// Storage path.
    pub path: PathBuf,
    /// Whether this identity is selected by `auth.json`.
    pub active: bool,
    /// Whether this identity entry exists but is unusable.
    pub broken: bool,
}

fn validate_identity_slug(value: &str) -> Result<(), Error> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(Error::InvalidIdentitySlug {
            slug: value.to_owned(),
        });
    };
    if !first.is_ascii_alphanumeric()
        || !chars.all(|char| char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-'))
    {
        return Err(Error::InvalidIdentitySlug {
            slug: value.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Error, IdentitySlug};

    #[test]
    fn identity_slug_validation_accepts_portable_slugs() {
        for slug in ["personal", "OpenAI-Work", "org.dev", "test_2", "work.json"] {
            assert!(IdentitySlug::try_from(slug).is_ok());
        }
    }

    #[test]
    fn identity_slug_validation_rejects_paths_and_shellish_slugs() {
        for slug in ["", "-prod", "my work", "../auth", "work/main", "work\\main"] {
            assert!(matches!(
                IdentitySlug::try_from(slug),
                Err(Error::InvalidIdentitySlug { .. })
            ));
        }
    }
}
