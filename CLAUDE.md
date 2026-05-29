# CLAUDE.md — target-cli

Guidance for AI agents working in this repo. Part of the
[baby-registry](https://github.com/piekstra/baby-registry) ecosystem; the shared
mission and rules live in that repo's `CLAUDE.md`.

## What this is

A Rust CLI over Target's **public RedSky API** for product search and lookup. No
credentials, no auth, no state. Its job is to turn a query (often produced by an
agent identifying a product from a photo) into a **canonical Target product URL**
that `babylist-cli add` can consume.

## Build / test / run

```sh
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all

./target/release/target-cli search "convertible car seat" --json
./target/release/target-cli get 80179068 --json
```

## Layout

- `src/cli.rs` — clap arg definitions.
- `src/client.rs` — RedSky HTTP client + the baked-in default public key.
- `src/product.rs` — the normalized `Product` and defensive JSON parsing (+ unit tests).
- `src/output.rs` — human vs `--json` rendering.
- `src/error.rs` — error type → exit-code mapping.
- `src/main.rs` — wiring + dispatch.

## When chaining as an agent

- Use `--json`. stdout is pure JSON; stderr is diagnostics.
- `search` returns an array; `get` returns one object.
- Respect exit codes (see README): `4` = not found, `5` = network, `6` = rate-limited.
- The field you almost always want next is `.url` (or `.[].url`).

## Gotchas

- RedSky responses **drift**; parsing is best-effort beyond `tcin`. If a field
  goes missing, fix the path in `product.rs` and add a test — don't make it required.
- PDP returns HTTP `206`; that's success. A bad TCIN returns `404` → exit `4`.
- If the public key stops working, it rotated — pass `--key` / `$TARGET_REDSKY_KEY`.
- Be polite: this targets a public API at human scale. No aggressive looping.
