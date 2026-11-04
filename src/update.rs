//! `tgt self-update`, through the family updater (`pk-cli-selfupdate`). Release assets are named
//! `tgt-<target-triple>.tar.gz`; the triple is baked in by `build.rs`.

pub use pk_cli_selfupdate::{SelfUpdateArgs, Updater};

/// The GitHub repository releases are pulled from.
pub const REPO: &str = "piekstra/target-cli";

pub fn updater() -> Updater {
    Updater {
        repo: REPO.into(),
        binary: "tgt".into(),
        target: env!("BUILD_TARGET").into(),
        current: env!("CARGO_PKG_VERSION").into(),
    }
}
