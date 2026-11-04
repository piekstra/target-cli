//! Thin binary: parse arguments and dispatch into the library's `commands`.

use clap::Parser;

use target_cli::cli::{AuthCmd, Cli, Command};
use target_cli::commands::{self, Ctx};
use target_cli::error::AppError;
use target_cli::mcp::{Live, TgtTools};

fn run(cli: &Cli) -> Result<(), AppError> {
    // None of these reaches Target or needs the config to be readable. `mcp` reads the config on
    // each call, so listing its tools reads nothing.
    match &cli.command {
        Command::Completions { shell } => return commands::completions::run(*shell),
        Command::Mcp(args) => return pk_cli_mcp::run(args, TgtTools::new(Live)),
        Command::Info => return commands::info::run(),
        Command::Auth(AuthCmd::Status) => return commands::auth::status(cli.common.json),
        Command::Config(cmd) => {
            return commands::config::run(cli.common.json, cli.common.quiet, cmd)
        }
        Command::SelfUpdate(args) => return commands::self_update::run(&cli.common, args),
        Command::Search { .. } | Command::Get { .. } => {}
    }

    let ctx = Ctx::new(cli)?;
    match &cli.command {
        Command::Search { query, limit } => commands::search::run(&ctx, query, *limit),
        Command::Get { tcin } => commands::get::run(&ctx, tcin),
        _ => unreachable!("handled above"),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        std::process::exit(pk_cli_core::output::fail(&e, cli.common.json));
    }
}
