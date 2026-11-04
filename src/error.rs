//! Error type shared with the CLI family: `pk_cli_core::CliError` carries the stable exit-code
//! contract (0 to 6) and the `--json` error envelope. RedSky failures map to `Upstream` (exit 5).

pub use pk_cli_core::CliError as AppError;
