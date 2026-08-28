use std::{fmt, fs};

use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use serde_json::Value;

use crate::{Error, Identity};

/// Displayable account details stored in an identity's ID token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityDetails {
    /// Account display name.
    pub display_name: Option<String>,
    /// Account email address.
    pub email: Option<String>,
}

impl fmt::Display for IdentityDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.display_name, &self.email) {
            (Some(display_name), Some(email)) => write!(f, "{display_name} <{email}>"),
            (Some(display_name), None) => f.write_str(display_name),
            (None, Some(email)) => write!(f, "<{email}>"),
            (None, None) => Ok(()),
        }
    }
}

impl Identity {
    /// Read displayable account details from this identity's ID token.
    ///
    /// Invalid auth JSON, malformed tokens, and tokens without displayable claims return `None`.
    ///
    /// # Errors
    ///
    /// Returns an error when the identity file cannot be read.
    pub fn read_details(&self) -> Result<Option<IdentityDetails>, Error> {
        read_auth_details(&self.path)
    }
}

pub fn read_auth_details(path: &std::path::Path) -> Result<Option<IdentityDetails>, Error> {
    let auth = fs::read(path).map_err(|source| Error::Io {
        action: "read auth details",
        path: path.to_path_buf(),
        source,
    })?;
    Ok(parse_auth_details(&auth))
}

fn parse_auth_details(auth: &[u8]) -> Option<IdentityDetails> {
    let auth: Value = serde_json::from_slice(auth).ok()?;
    let token = auth.pointer("/tokens/id_token")?.as_str()?;
    let mut segments = token.split('.');
    segments.next()?;
    let payload = segments.next()?;
    segments.next()?;
    if segments.next().is_some() {
        return None;
    }
    let payload = BASE64_URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: Value = serde_json::from_slice(&payload).ok()?;
    let details = IdentityDetails {
        display_name: clean_claim(claims.get("name")),
        email: clean_claim(claims.get("email")),
    };
    (details.display_name.is_some() || details.email.is_some()).then_some(details)
}

fn clean_claim(value: Option<&Value>) -> Option<String> {
    let value: String = value?
        .as_str()?
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{Identity, IdentityDetails, IdentityName};

    use super::parse_auth_details;

    #[test]
    fn identity_reads_name_and_email_from_id_token() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cam-auth-{unique}.json"));
        fs::write(
            &path,
            r#"{"tokens":{"id_token":"header.eyJuYW1lIjoiRXhhbXBsZSBVc2VyIiwiZW1haWwiOiJ0aGUudXNlckBnbWFpbC5jb20ifQ.signature"}}"#,
        )
        .unwrap();
        let identity = Identity {
            name: IdentityName::try_from("personal").unwrap(),
            path: path.clone(),
            active: false,
            broken: false,
        };

        let details = identity.read_details().unwrap().unwrap();

        assert_eq!(
            details,
            IdentityDetails {
                display_name: Some("Example User".to_owned()),
                email: Some("the.user@gmail.com".to_owned()),
            }
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn details_display_available_claims() {
        let cases = [
            (
                IdentityDetails {
                    display_name: Some("Example User".to_owned()),
                    email: Some("the.user@gmail.com".to_owned()),
                },
                "Example User <the.user@gmail.com>",
            ),
            (
                IdentityDetails {
                    display_name: Some("Example User".to_owned()),
                    email: None,
                },
                "Example User",
            ),
            (
                IdentityDetails {
                    display_name: None,
                    email: Some("the.user@gmail.com".to_owned()),
                },
                "<the.user@gmail.com>",
            ),
        ];

        for (details, expected) in cases {
            assert_eq!(details.to_string(), expected);
        }
    }

    #[test]
    fn identity_details_remove_terminal_control_characters() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cam-auth-controls-{unique}.json"));
        fs::write(
            &path,
            r#"{"tokens":{"id_token":"header.eyJuYW1lIjoiIEV4YW1wbGVcblVzZXJcdTAwMWIgIiwiZW1haWwiOiIgdXNlckBleGFtcGxlLmNvbVxuIn0.signature"}}"#,
        )
        .unwrap();
        let identity = Identity {
            name: IdentityName::try_from("personal").unwrap(),
            path: path.clone(),
            active: false,
            broken: false,
        };

        let details = identity.read_details().unwrap().unwrap();

        assert_eq!(details.display_name.as_deref(), Some("ExampleUser"));
        assert_eq!(details.email.as_deref(), Some("user@example.com"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn malformed_auth_details_are_absent() {
        let cases: [&[u8]; 4] = [
            b"not json",
            br#"{"tokens":{}}"#,
            br#"{"tokens":{"id_token":"not-a-jwt"}}"#,
            br#"{"tokens":{"id_token":"header.invalid.signature"}}"#,
        ];

        for auth in cases {
            assert_eq!(parse_auth_details(auth), None);
        }
    }
}
