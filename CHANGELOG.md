# Changelog

All notable changes to `tgt` are documented here. Newest first. The project follows
[SemVer](https://semver.org/); until 1.0, minors carry the breaking-change signal.

## 0.2.0 — 2026-11-03

Adopts cli-common v0.28.0 and the family spec, piekstra-cli/1.

- **Breaking:** `--json` prints one tagged document. `tgt search --json` is
  `{"schema": "tgt-search/v1", "query", "store", "returned", "products": [...]}`, where it was a bare
  array, so `jq '.[].url'` becomes `jq '.products[].url'`. `tgt get --json` gains
  `"schema": "tgt-product/v1"`. A failure under `--json` is one `{"error": {"code", "message"}}` document on stdout.
- **Breaking:** rate limiting exits 5, upstream, where it exited 6. 6 is the family's "confirmation
  required".
- `tgt mcp`: two read-only tools for Claude, `tgt_search` and `tgt_product`. They run over stdio for
  Claude Code and Claude Desktop, and with `--http` on 127.0.0.1 behind a bearer token for Claude on
  the web and mobile through your own Cloudflare Worker and tunnel. No tool can buy anything.
- `info`, `auth status`, `config path|show|set|unset`, `self-update` and `completions`.
- `search --limit`, up to 200, reads on across RedSky's pages of 28. `-n` and `--count` still work.
  It used to cap at 28 without saying so.
- `--store` defaults to the `store` config key. `$TGT_STORE` sets it too.
- A TCIN or a store id that is not digits is refused before any request, with exit 2.
- A search answer with no product list is an upstream error, exit 5, where it read as "no products",
  exit 4. An answer with no `data` for a product does the same.
- Target refusing the client (HTTP 403 or 435) is reported as a refusal. This tool does not try to
  get around it.
- Releases: tag-triggered draft-then-publish with `.mcpb`, macOS, Linux and Windows assets, and a
  smoke run of each binary a runner can execute. These are this repo's first releases.
- `rust-version` is now 1.89, which `pk-cli-auth` needs.
