//! `info`: machine-readable capability discovery (`cli-info/v1`, SPEC v1 §1.6).

use pk_cli_core::info::{AuthInfo, CliInfo};
use pk_cli_core::output;

use crate::error::AppError;
use crate::update::REPO;

pub fn run() -> Result<(), AppError> {
    let info = CliInfo::new(
        "tgt",
        env!("CARGO_PKG_VERSION"),
        &format!("https://github.com/{REPO}"),
        AuthInfo {
            required: false,
            method: "none".into(),
            login_hint: None,
        },
        &["search", "get", "mcp"],
    );
    output::json(&serde_json::to_value(&info).unwrap_or_default());
    Ok(())
}
