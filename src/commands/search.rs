//! `tgt search`.

use pk_cli_core::output;
use serde_json::{json, Value};

use crate::commands::{print_product, Ctx, DEFAULT_LIMIT};
use crate::config::checked_limit;
use crate::error::AppError;
use crate::product::{walk, Product, Stopped};

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
    let walked = walk(
        |offset, count| ctx.api.search_page(keyword, offset, count),
        0,
        limit,
        None,
    )?;
    let failure = match walked.stopped {
        Some(Stopped::Failed(e)) => Some(e),
        Some(Stopped::OutOfTime) | None => None,
    };
    let products: Vec<Product> = walked.found.into_iter().map(|(_, p)| p).collect();
    if products.is_empty() {
        return Err(AppError::NotFound(format!("no products for {keyword:?}")));
    }
    output::emit(
        ctx.json,
        "tgt-search/v1",
        document(keyword, &ctx.store, &products, failure.as_ref()),
        |_: &Value| {
            for (i, p) in products.iter().enumerate() {
                print_product(p, Some(i + 1));
            }
        },
    );
    if let Some(e) = failure {
        // The document above is this run's one document, so this does not return the error,
        // which would print a second.
        eprintln!("error: {e}");
        eprintln!(
            "error: the search stopped there, so the {} products above may not be all of them",
            products.len()
        );
        std::process::exit(e.exit_code());
    }
    Ok(())
}

/// The search's one document. `stopped` says a partial list is partial.
fn document(keyword: &str, store: &str, products: &[Product], failure: Option<&AppError>) -> Value {
    let mut doc = json!({
        "query": keyword,
        "store": store,
        "returned": products.len(),
        "products": products,
    });
    if let Some(e) = failure {
        doc["stopped"] = json!(e.to_string());
    }
    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_partial_list_says_so_and_a_whole_one_says_nothing() {
        let whole = document("lamp", "1", &[], None);
        assert!(whole.get("stopped").is_none(), "{whole}");
        let e = AppError::Upstream("HTTP 500".into());
        let partial = document("lamp", "1", &[], Some(&e));
        assert!(partial["stopped"].as_str().unwrap().contains("HTTP 500"));
    }
}
