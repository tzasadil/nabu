use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

/// One parse path for every TOML file => errors always name the file and carry the parser's snippet.
pub fn parse_toml<T: DeserializeOwned>(path: &str, text: &str) -> Result<T> {
    toml::from_str(text).map_err(|err| {
        Error::new(
            format!("{path}: {}", err.to_string().trim_end()),
            format!("fix the TOML in {path}"),
        )
    })
}
