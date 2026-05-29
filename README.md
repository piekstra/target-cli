# target-cli

Search and look up Target products from the command line via Target's public
**RedSky** API. No account, no API key, no scraping — RedSky is a public endpoint
that target.com's own front end uses, so this is just a polite client over it.

Built to be **human- and agent-friendly**: every command has a `--json` mode and
stable exit codes, so a person or an AI agent can chain it into a pipeline. It's
part of the [baby-registry](https://github.com/piekstra/baby-registry) toolkit —
its output URLs drop straight into `babylist-cli add`.

## Install

```sh
cargo build --release        # binary at ./target/release/tgt
# or
cargo install --path .       # installs the `tgt` binary onto your PATH
```

> The crate/repo is `target-cli`, but the binary is **`tgt`** (short and quick to
> type — it's also Target's ticker).

Requires a recent stable Rust toolchain.

## Usage

### Search

```sh
$ tgt search "convertible car seat" -n 3
1. Graco TriRide 3-in-1 Convertible Car Seat - Kipling  [$199.99]
   brand: Graco
   tcin:  80179068
   url:   https://www.target.com/p/graco-triride-3-in-1-convertible-car-seat-kipling/-/A-80179068

2. Graco SlimFit 3-in-1 Convertible Car Seat - Jarret ...  [$229.99]
   brand: Graco
   tcin:  92620482
   url:   https://www.target.com/p/.../-/A-92620482
...
```

### Look up a product by TCIN

```sh
$ tgt get 80179068
Graco TriRide 3-in-1 Convertible Car Seat  [$199.99]
   brand: Graco
   tcin:  82286564
   url:   https://www.target.com/p/graco-triride-3-in-1-convertible-car-seat/-/A-82286564
```

### JSON output (for scripting / agents)

```sh
# Just the canonical product URLs, ready to pipe into babylist-cli:
$ tgt search "graco stroller" -n 5 --json | jq -r '.[].url'

# Full structured record:
$ tgt get 80179068 --json
{
  "tcin": "82286564",
  "title": "Graco TriRide 3-in-1 Convertible Car Seat",
  "brand": "Graco",
  "price": "$199.99",
  "url": "https://www.target.com/p/.../-/A-82286564",
  "image": "https://target.scene7.com/is/image/Target/GUEST_..."
}
```

In `--json` mode the JSON document is the only thing on **stdout**; diagnostics
go to **stderr**, so `| jq` is always safe.

## Global flags

| Flag | Description |
|------|-------------|
| `--json` | Machine-readable JSON on stdout |
| `-v, --verbose` | Extra diagnostics on stderr |
| `-q, --quiet` | Suppress non-error stderr |
| `--no-color` | Disable ANSI color (reserved) |
| `--store <ID>` | Pricing/availability store id (default `3991`) |
| `--key <KEY>` | Override the RedSky key (or `$TARGET_REDSKY_KEY`) if the default rotates |
| `-V, --version` | Print version |
| `-h, --help` | Help |

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | success |
| `1` | generic error |
| `2` | usage error (bad args) |
| `4` | not found (no matching product / bad TCIN) |
| `5` | network / upstream error |
| `6` | rate-limited (HTTP 429) |

## How it works

Two public RedSky aggregation endpoints:

- **Search** → `…/v1/web/plp_search_v2` (`data.search.products[]`)
- **Detail** → `…/v1/web/pdp_client_v1` (`data.product`)

The client sends a descriptive User-Agent, uses a 20s timeout, and normalizes the
(drift-prone) responses into a small stable record. If the baked-in public key
ever stops working, pass a fresh one with `--key` / `$TARGET_REDSKY_KEY` — no code
change needed.

## Scope & disclaimer

A personal-use tool for looking up publicly available product data at human
scale. It uses no credentials and stores nothing. Please use it within Target's
Terms of Service. Not affiliated with or endorsed by Target.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. See [`CONTRIBUTING.md`](CONTRIBUTING.md).
