//! Non-secret preferences, stored by the family config crate at
//! `${XDG_CONFIG_HOME:-~/.config}/tgt/config.json`. Nothing here is a secret.

use pk_cli_config::ConfigStore;
use serde::{Deserialize, Serialize};

use crate::client::MAX_LIMIT;
use crate::error::AppError;
use crate::product::checked_store;

/// Config directory / store name. Matches the binary name, per SPEC v1.
pub const APP: &str = "tgt";

/// The config keys `tgt config set` accepts, for validation and error text.
pub const KEYS: &[&str] = &["store", "limit"];

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The store whose prices and availability to read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,

    /// Default `--limit` for searches when the flag is not given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

pub fn store() -> ConfigStore {
    ConfigStore::new(APP)
}

/// Load the saved preferences (a missing file is not an error).
pub fn load() -> Result<Config, AppError> {
    store().load()
}

/// Apply `key=value`, validating both.
pub fn set(cfg: &mut Config, key: &str, value: &str) -> Result<(), AppError> {
    match key {
        "store" => cfg.store = Some(checked_store(value)?),
        "limit" => {
            let n: u32 = value.parse().map_err(|_| {
                AppError::Usage(format!("limit must be a whole number, got `{value}`"))
            })?;
            cfg.limit = Some(checked_limit(n)?);
        }
        other => return Err(unknown_key(other)),
    }
    Ok(())
}

/// A search's limit, from a flag or the config.
pub fn checked_limit(n: u32) -> Result<u32, AppError> {
    if (1..=MAX_LIMIT).contains(&n) {
        Ok(n)
    } else {
        Err(AppError::Usage(format!(
            "limit must be between 1 and {MAX_LIMIT}, got {n}"
        )))
    }
}

/// Clear `key`. Returns `true` if it had been set.
pub fn unset(cfg: &mut Config, key: &str) -> Result<bool, AppError> {
    let had = match key {
        "store" => cfg.store.take().is_some(),
        "limit" => cfg.limit.take().is_some(),
        other => return Err(unknown_key(other)),
    };
    Ok(had)
}

fn unknown_key(key: &str) -> AppError {
    AppError::Usage(format!(
        "unknown config key `{key}` (known: {})",
        KEYS.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_and_unsets_known_keys() {
        let mut cfg = Config::default();
        set(&mut cfg, "store", " 1234 ").unwrap();
        assert_eq!(cfg.store.as_deref(), Some("1234"));
        assert!(unset(&mut cfg, "store").unwrap());
        assert!(!unset(&mut cfg, "store").unwrap());
        set(&mut cfg, "limit", "25").unwrap();
        assert_eq!(cfg.limit, Some(25));
    }

    #[test]
    fn rejects_bad_values_and_unknown_keys() {
        let mut cfg = Config::default();
        for (key, value) in [
            ("store", "not-a-store"),
            ("limit", "abc"),
            ("limit", "0"),
            ("limit", "201"),
            ("nope", "x"),
        ] {
            assert_eq!(
                set(&mut cfg, key, value).unwrap_err().exit_code(),
                2,
                "{key}={value}"
            );
        }
        assert_eq!(unset(&mut cfg, "nope").unwrap_err().exit_code(), 2);
    }

    #[test]
    fn empty_config_serializes_to_an_empty_object() {
        assert_eq!(serde_json::to_string(&Config::default()).unwrap(), "{}");
    }
}
