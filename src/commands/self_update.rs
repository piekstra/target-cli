//! `self-update`: update tgt to the latest GitHub release, through the family updater.

use pk_cli_core::CommonArgs;
use pk_cli_selfupdate::SelfUpdateArgs;

use crate::error::AppError;
use crate::update;

pub fn run(common: &CommonArgs, args: &SelfUpdateArgs) -> Result<(), AppError> {
    update::updater().run_noting(
        args,
        common.json,
        common.quiet,
        &pk_cli_mcp::restart_note("tgt"),
    )
}
