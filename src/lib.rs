//! `target_cli`, the library behind the `tgt` CLI: the RedSky client, the product model and
//! the MCP tools. `main.rs` is a thin binary that parses arguments and dispatches into
//! [`commands`].

pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod mcp;
pub mod product;
pub mod update;
pub mod version;
