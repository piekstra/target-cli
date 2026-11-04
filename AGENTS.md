# AGENTS.md: target-cli

Canonical agent entrypoint for this repo. `CLAUDE.md` is a one-line pointer here.

## What this is

A Rust CLI (`tgt`) over **RedSky**, the aggregation API target.com's own front end calls, for
product search and lookup. **No credential, no `auth login`.** It sends the key target.com's own
client-side code carries, which is public (`.gitleaks.toml` allowlists it). `auth status` reports
`required: false` because SPEC v1 asks every family member for it. The keychain holds one thing,
the bearer token of `mcp --http`, and only after `tgt mcp --new-token`.

RedSky has no published contract and sits behind a bot wall. See `docs/api.md`.

## Build / test / run

```sh
make verify       # fmt-check, lint, test, smoke: what CI runs
make release      # ./target/release/tgt

./target/release/tgt search "desk lamp" --json
./target/release/tgt get 12345678 --json
```

## Layout

| Path | What |
|---|---|
| `src/main.rs` | Thin binary: parse, dispatch, map errors to exit codes |
| `src/cli.rs` | The clap tree |
| `src/client.rs` | `RedSky`, the HTTP client |
| `src/product.rs` | `Product`, what an answer means, `walk` across pages, argument shapes |
| `src/commands/` | One module per command |
| `src/mcp.rs`, `src/mcp/tests.rs` | `tgt mcp`: `tgt_search` and `tgt_product` |
| `src/config.rs` | Settings: `store`, `limit`, `key` (public) |
| `tests/` | Offline: command surface, MCP over stdio and HTTP, packaging |
| `packaging/mcpb/manifest.json` | The Claude Desktop extension |

## Rules

- **Do not work around Target's bot wall.** No browser user agent, no borrowed cookies, no
  fingerprint tricks, no proxy. A refusal (HTTP 401, 403 or 435) is reported as one, with
  `Reason::ClientBlocked`. Whether to keep this CLI on RedSky is the owner's question.
- **Tools only read.** No tool, argument or code path may buy, add to a cart, or touch an account.
  There is no such thing in `RedSky`, and it stays that way.
- **A changed answer fails.** RedSky's schema drifts. A missing list is an upstream error and never
  "no products", while a present, empty one is a real answer. Fields beyond `tcin` are best-effort.
- **Check an argument's shape before a request.** A TCIN and a store are digits, and a cursor's key
  is a row number. Each reaches a query string.
- **A failure part way keeps what was read,** and says the list is partial. Do not retry within a
  search, and keep the pause between pages.
- **A price is Target's text**, which can be a range. Do not parse it into a number.
- **Tests are offline** and read no keychain. Fixture values are obvious dummies
  (`tests/fixtures/README.md`). No real product, price or store belongs in a fixture or a doc.
- **No secret goes in a config file, and no secret on the command line.** `--key` is the exception
  because that key is public.

## The CLI family & cli-common

`tgt` conforms to **piekstra-cli/1**, defined in
[cli-common](https://github.com/piekstra/cli-common)'s `DESIGN.md` and `conformance.md`. Shared
behavior comes from the `pk-cli-*` crates, tag-pinned (currently **v0.28.0**) and all on one tag.
Reuse before writing. A release is a pushed `v*` tag and nothing else.
