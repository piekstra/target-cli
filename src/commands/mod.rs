//! Command implementations. Each function takes a shared [`Ctx`] plus its parsed arguments.

pub mod auth;
pub mod completions;
pub mod config;
pub mod get;
pub mod info;
pub mod search;
pub mod self_update;

use crate::cli::Cli;
use crate::client::{RedSky, DEFAULT_STORE};
use crate::config::Config;
use crate::error::AppError;
use crate::product::checked_store;

/// Fallback for `--limit` when neither the flag nor the config sets one.
pub const DEFAULT_LIMIT: u32 = 10;

/// What a command that reaches Target needs. Arguments are checked here, before any request.
pub struct Ctx {
    pub api: RedSky,
    pub cfg: Config,
    pub store: String,
    pub json: bool,
    pub verbose: bool,
    pub quiet: bool,
}

impl Ctx {
    pub fn new(cli: &Cli) -> Result<Self, AppError> {
        let cfg = crate::config::load()?;
        let store = checked_store(
            cli.store
                .as_deref()
                .or(cfg.store.as_deref())
                .unwrap_or(DEFAULT_STORE),
        )?;
        Ok(Ctx {
            api: RedSky::new(cli.key.clone(), store.clone())?,
            cfg,
            store,
            json: cli.common.json,
            verbose: cli.common.verbose,
            quiet: cli.common.quiet,
        })
    }

    /// Emit a diagnostic line (stderr) when `--verbose` and not `--quiet`.
    pub fn log(&self, msg: &str) {
        if self.verbose && !self.quiet {
            eprintln!("{msg}");
        }
    }
}

/// One product for a person to read. `index` numbers it in a list.
pub fn print_product(p: &crate::product::Product, index: Option<usize>) {
    let number = index.map(|i| format!("{i}. ")).unwrap_or_default();
    println!(
        "{number}{}  [{}]",
        p.title,
        p.price.as_deref().unwrap_or("-")
    );
    if let Some(brand) = p.brand.as_deref().filter(|b| !b.is_empty()) {
        println!("   brand: {brand}");
    }
    println!("   tcin:  {}", p.tcin);
    if let Some(url) = &p.url {
        println!("   url:   {url}");
    }
    println!();
}
