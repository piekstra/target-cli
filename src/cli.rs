//! Command-line surface. Conforms to **piekstra-cli/1** (cli-common's `DESIGN.md`): the global
//! flags come from the shared [`CommonArgs`], and the standard `auth` / `config` / `self-update`
//! / `completions` / `info` commands are spelled as the spec requires.

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use pk_cli_core::CommonArgs;
pub use pk_cli_selfupdate::SelfUpdateArgs;

/// Search and look up Target products through the RedSky API behind target.com.
///
/// No account and no credentials. Unofficial; not affiliated with Target.
#[derive(Parser, Debug)]
#[command(name = "tgt", version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Store whose prices and availability to read. Defaults to the `store` config key.
    #[arg(long, global = true, value_name = "ID", env = "TGT_STORE")]
    pub store: Option<String>,

    /// RedSky's public key, if the built-in one rotates. It is target.com's own front-end key
    /// and not a credential.
    #[arg(long, global = true, value_name = "KEY", env = "TARGET_REDSKY_KEY")]
    pub key: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search products by keyword.
    Search {
        /// Search terms, joined with spaces.
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// Most results to return. Defaults to the `limit` config key, or 10.
        #[arg(short = 'n', long = "limit", visible_alias = "count", value_name = "N")]
        limit: Option<u32>,
    },

    /// Look up one product by TCIN, Target's product number.
    Get {
        /// Target product id (TCIN).
        tcin: String,
    },

    /// Serve read-only tools to Claude and other MCP clients over stdio, or with --http on
    /// 127.0.0.1 for your own connector's tunnel.
    Mcp(pk_cli_mcp::McpArgs),

    /// Machine-readable capability discovery (cli-info/v1).
    Info,

    /// Report credential state. This tool needs none.
    #[command(subcommand)]
    Auth(AuthCmd),

    /// Non-secret settings.
    #[command(subcommand)]
    Config(ConfigCmd),

    /// Update tgt to the latest release from GitHub.
    #[command(name = "self-update")]
    SelfUpdate(SelfUpdateArgs),

    /// Print a shell completion script (e.g. `tgt completions zsh`).
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthCmd {
    /// Report credential state (auth-status/v1).
    Status,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Print the resolved config file path.
    Path,
    /// Show the effective configuration.
    Show,
    /// Set a config key, e.g. `tgt config set store 1234`.
    Set { key: String, value: String },
    /// Remove a config key.
    Unset { key: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn search_joins_its_terms_and_leaves_the_limit_to_the_config() {
        let cli = Cli::parse_from(["tgt", "search", "desk", "lamp"]);
        match cli.command {
            Command::Search { query, limit } => {
                assert_eq!(query, ["desk", "lamp"]);
                assert_eq!(limit, None);
            }
            other => panic!("expected search, got {other:?}"),
        }
    }

    #[test]
    fn the_old_count_flag_still_reads() {
        for args in [
            ["tgt", "search", "lamp", "--count", "3"],
            ["tgt", "search", "lamp", "--limit", "3"],
            ["tgt", "search", "lamp", "-n", "3"],
        ] {
            match Cli::parse_from(args).command {
                Command::Search { limit, .. } => assert_eq!(limit, Some(3), "{args:?}"),
                other => panic!("expected search, got {other:?}"),
            }
        }
    }

    #[test]
    fn json_is_global_and_works_after_the_subcommand() {
        assert!(Cli::parse_from(["tgt", "get", "1", "--json"]).common.json);
    }
}
