mod cli;
mod client;
mod error;
mod output;
mod product;

use clap::Parser;

use cli::{Cli, Command};
use client::{RedSky, DEFAULT_KEY};
use error::AppError;
use product::Product;

fn run(cli: Cli) -> Result<(), AppError> {
    let key = cli.key.clone().unwrap_or_else(|| DEFAULT_KEY.to_string());
    let api = RedSky::new(key, cli.store.clone())?;

    match &cli.command {
        Command::Search { query, count } => {
            let keyword = query.join(" ");
            if cli.verbose && !cli.quiet {
                eprintln!("searching Target for {keyword:?} (store {})", cli.store);
            }
            let body = api.search(&keyword, *count)?;
            let products: Vec<Product> = body
                .pointer("/data/search/products")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(Product::from_node).collect())
                .unwrap_or_default();
            if products.is_empty() {
                return Err(AppError::NotFound(format!("no products for {keyword:?}")));
            }
            output::print_search(&products, cli.json);
        }
        Command::Get { tcin } => {
            if cli.verbose && !cli.quiet {
                eprintln!("looking up tcin {tcin} (store {})", cli.store);
            }
            let body = api.product(tcin)?;
            let node = body
                .pointer("/data/product")
                .ok_or_else(|| AppError::NotFound(format!("no product with tcin {tcin}")))?;
            let product = Product::from_node(node)
                .ok_or_else(|| AppError::NotFound(format!("no product with tcin {tcin}")))?;
            output::print_one(&product, cli.json);
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let quiet = cli.quiet;
    if let Err(e) = run(cli) {
        if !quiet {
            eprintln!("error: {e}");
        }
        std::process::exit(e.exit_code());
    }
}
