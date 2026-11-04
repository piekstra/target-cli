//! `tgt search`.

use pk_cli_core::output;
use serde_json::{json, Value};

use crate::commands::{print_product, Ctx, DEFAULT_LIMIT};
use crate::config::checked_limit;
use crate::error::AppError;
use crate::product::{walk, Product};

pub fn run(ctx: &Ctx, query: &[String], limit: Option<u32>) -> Result<(), AppError> {
    let keyword = query.join(" ");
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Err(AppError::Usage("search terms must not be blank".into()));
    }
    let limit = checked_limit(limit.or(ctx.cfg.limit).unwrap_or(DEFAULT_LIMIT))?;
    ctx.log(&format!(
        "searching Target for {keyword:?} (store {})",
        ctx.store
    ));
    let found = walk(
        |offset, count| ctx.api.search_page(keyword, offset, count),
        0,
        limit,
    )?;
    let products: Vec<Product> = found.into_iter().map(|(_, p)| p).collect();
    if products.is_empty() {
        return Err(AppError::NotFound(format!("no products for {keyword:?}")));
    }
    output::emit(
        ctx.json,
        "tgt-search/v1",
        json!({
            "query": keyword,
            "store": ctx.store,
            "returned": products.len(),
            "products": products,
        }),
        |_: &Value| {
            for (i, p) in products.iter().enumerate() {
                print_product(p, Some(i + 1));
            }
        },
    );
    Ok(())
}
