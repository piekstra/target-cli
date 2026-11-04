//! Black-box checks of the `tgt` binary's offline surface. Each run gets its own config dir, so
//! no test reads the real config, and nothing here reaches Target: every failing call is refused
//! on its arguments before a request.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn tgt(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tgt"));
    cmd.env("XDG_CONFIG_HOME", home)
        .env("HOME", home)
        .env_remove("TGT_STORE")
        .env_remove("TARGET_REDSKY_KEY")
        .env_remove("NO_COLOR");
    cmd
}

/// A fresh dir per call; the counter keeps parallel tests apart.
fn tmp() -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "tgt-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(home: &Path, args: &[&str]) -> Output {
    tgt(home).args(args).output().unwrap()
}

fn stdout_json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)))
}

#[test]
fn every_command_has_help() {
    let home = tmp();
    for args in [
        vec!["--help"],
        vec!["search", "--help"],
        vec!["get", "--help"],
        vec!["mcp", "--help"],
        vec!["info", "--help"],
        vec!["auth", "--help"],
        vec!["auth", "status", "--help"],
        vec!["config", "--help"],
        vec!["config", "set", "--help"],
        vec!["self-update", "--help"],
        vec!["completions", "--help"],
    ] {
        assert!(run(&home, &args).status.success(), "{args:?}");
    }
}

#[test]
fn no_color_in_the_environment_never_fails_a_command() {
    // Any non-empty NO_COLOR disables color (no-color.org); parsed as a bool, the conventional
    // NO_COLOR=1 would be a usage error.
    let home = tmp();
    for v in ["1", "true", "0", ""] {
        let out = tgt(&home)
            .env("NO_COLOR", v)
            .args(["completions", "bash"])
            .output()
            .unwrap();
        assert!(out.status.success(), "NO_COLOR={v:?}");
    }
}

#[test]
fn info_names_the_binary_and_needs_no_credential() {
    let out = run(&tmp(), &["info"]);
    assert!(out.status.success());
    let v = stdout_json(&out);
    assert_eq!(v["schema"], "cli-info/v1");
    assert_eq!(v["name"], "tgt");
    assert_eq!(v["auth"]["required"], false);
}

#[test]
fn auth_status_says_none_is_required() {
    let out = run(&tmp(), &["auth", "status", "--json"]);
    assert!(out.status.success());
    let v = stdout_json(&out);
    assert_eq!(v["schema"], "auth-status/v1");
    assert_eq!(v["required"], false);
}

#[test]
fn bad_arguments_exit_2_before_any_request() {
    let home = tmp();
    for args in [
        vec!["search"],
        vec!["search", "  "],
        vec!["search", "lamp", "--limit", "0"],
        vec!["search", "lamp", "--limit", "201"],
        vec!["search", "lamp", "--store", "not-a-store"],
        vec!["get"],
        vec!["get", "not-a-number"],
        vec!["get", "1", "--store", "not-a-store"],
        vec!["config", "set", "store", "not-a-store"],
        vec!["config", "set", "limit", "0"],
        vec!["config", "set", "nope", "1"],
        vec!["config", "unset", "nope"],
        vec!["nope"],
    ] {
        let out = run(&home, &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn a_failure_under_json_is_one_error_document() {
    let out = run(&tmp(), &["get", "not-a-number", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v = stdout_json(&out);
    assert_eq!(v["error"]["code"], "usage");
    assert!(v["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("TCIN"));
}

#[test]
fn the_config_round_trips_and_its_path_is_under_the_config_dir() {
    let home = tmp();
    let path = run(&home, &["config", "path"]);
    assert!(path.status.success());
    let path = String::from_utf8_lossy(&path.stdout).to_string();
    assert!(path.trim().ends_with("tgt/config.json"), "{path}");

    assert!(run(&home, &["config", "set", "store", "1234"])
        .status
        .success());
    let shown = stdout_json(&run(&home, &["config", "show", "--json"]));
    assert_eq!(shown["store"], "1234");
    assert!(run(&home, &["config", "unset", "store"]).status.success());
    let shown = stdout_json(&run(&home, &["config", "show", "--json"]));
    assert!(shown.get("store").is_none(), "{shown}");
}

/// What repairs a broken config must still run with one.
#[test]
fn a_broken_config_stops_a_search_but_not_what_would_repair_it() {
    let home = tmp();
    let path = run(&home, &["config", "path"]);
    let path = String::from_utf8_lossy(&path.stdout).trim().to_string();
    std::fs::create_dir_all(Path::new(&path).parent().unwrap()).unwrap();
    std::fs::write(&path, "{not json").unwrap();

    assert!(run(&home, &["config", "path"]).status.success());
    assert!(run(&home, &["info"]).status.success());
    assert!(run(&home, &["config", "unset", "store"]).status.code() != Some(0));
    assert!(!run(&home, &["search", "lamp"]).status.success());
}

#[test]
fn completions_print_for_a_shell() {
    let out = run(&tmp(), &["completions", "zsh"]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("tgt"));
}
