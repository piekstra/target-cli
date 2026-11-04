//! `tgt get`.

use pk_cli_core::output;
use serde_json::Value;

use crate::commands::{print_product, Ctx};
use crate::error::AppError;
use crate::product::{checked_tcin, one_product};

pub fn run(ctx: &Ctx, tcin: &str) -> Result<(), AppError> {
    let tcin = checked_tcin(tcin)?;
    ctx.log(&format!("looking up tcin {tcin} (store {})", ctx.store));
    let product = one_product(&ctx.api.product(&tcin)?, &tcin)?;
    let dto = serde_json::to_value(&product).unwrap_or_default();
    output::emit(ctx.json, "tgt-product/v1", dto, |_: &Value| {
        print_product(&product, None);
    });
    Ok(())
}
