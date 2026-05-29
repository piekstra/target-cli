use clap::{Parser, Subcommand};

/// Search and look up Target products via the public RedSky API.
///
/// No account or credentials required — RedSky is a public endpoint.
#[derive(Parser, Debug)]
#[command(name = "tgt", version, about, long_about = None)]
pub struct Cli {
    /// Emit machine-readable JSON on stdout (diagnostics go to stderr).
    #[arg(long, global = true)]
    pub json: bool,

    /// Extra diagnostics on stderr (never secrets).
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error stderr output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable ANSI color (reserved; output is currently plain).
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Pricing/availability store id.
    #[arg(long, global = true, default_value = "3991")]
    pub store: String,

    /// RedSky public API key. Override if the built-in default rotates;
    /// falls back to $TARGET_REDSKY_KEY then a baked-in default.
    #[arg(long, global = true, env = "TARGET_REDSKY_KEY")]
    pub key: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search products by keyword.
    Search {
        /// Search terms (joined with spaces).
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// Maximum results to return (1-28).
        #[arg(short = 'n', long, default_value_t = 10)]
        count: u32,
    },

    /// Look up a single product by TCIN.
    Get {
        /// Target product id (TCIN).
        tcin: String,
    },
}
