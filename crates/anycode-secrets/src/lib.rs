//! Provider API keys in the OS credential store (PRD §22) — Keychain on macOS,
//! Credential Manager on Windows, Secret Service on Linux. A key never sits in
//! application state, a config file, or a log; it's read from here only at the moment
//! a request is signed, by the adapter that needs it — never handed to the renderer.

use keyring::Entry;

const SERVICE: &str = "dev.anycode.desktop";

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error(transparent)]
    Keyring(#[from] keyring::Error),
}

fn entry(provider_id: &str) -> Result<Entry, SecretError> {
    Entry::new(SERVICE, provider_id).map_err(Into::into)
}

pub fn set_api_key(provider_id: &str, key: &str) -> Result<(), SecretError> {
    entry(provider_id)?.set_password(key)?;
    Ok(())
}

pub fn get_api_key(provider_id: &str) -> Result<Option<String>, SecretError> {
    match entry(provider_id)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_api_key(provider_id: &str) -> Result<(), SecretError> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    /// The OS keychain is replaced by keyring's in-memory mock, so these run anywhere,
    /// CI included, without touching the user's real credentials.
    fn use_mock_store() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            keyring::set_default_credential_builder(keyring::mock::default_credential_builder())
        });
    }

    #[test]
    fn a_missing_key_reads_as_none_not_an_error() {
        use_mock_store();
        // "No key configured" is an ordinary state the UI shows, not a failure.
        assert_eq!(get_api_key("provider-with-no-key").unwrap(), None);
    }

    #[test]
    fn removing_a_key_that_was_never_set_is_not_an_error() {
        use_mock_store();
        // Disconnecting twice, or disconnecting a provider that was never connected,
        // must not surface as an error in Settings.
        delete_api_key("provider-with-no-key").unwrap();
    }

    #[test]
    fn setting_a_key_succeeds_against_the_store() {
        use_mock_store();
        set_api_key("provider-under-test", "sk-test-not-a-real-key").unwrap();
    }

    /// Round trip through the real OS credential store. Ignored by default: CI's Linux
    /// runner has no Secret Service, and a developer's run must opt in to writing (and
    /// then deleting) an entry in their own keychain. Do not combine with the mock tests
    /// in one process (`--include-ignored`): the mock builder is process-wide.
    ///
    /// `cargo test -p anycode-secrets -- --ignored`
    #[test]
    #[ignore = "writes to the real OS keychain"]
    fn round_trips_through_the_real_keychain() {
        let provider = "anycode-test-roundtrip";
        set_api_key(provider, "sk-test-not-a-real-key").unwrap();
        assert_eq!(
            get_api_key(provider).unwrap().as_deref(),
            Some("sk-test-not-a-real-key")
        );
        delete_api_key(provider).unwrap();
        assert_eq!(get_api_key(provider).unwrap(), None);
    }
}
