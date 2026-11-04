# target-cli

Search and look up Target products from the command line, through **RedSky**, the API target.com's
own front end calls. No account and no key to supply.

Every command has a `--json` mode and stable exit codes, so a person or an agent can chain it into a
pipeline. `tgt mcp` serves the same reads to Claude.

**Unofficial; not affiliated with or endorsed by Target.** RedSky has no published contract, and
Target's site turns away some automated clients. When it does, `tgt` says so (HTTP 401, 403 or 435,
exit 5) and does not try to get around it. On 2026-11-03 it answered that way to every request from the
machine this release was built on ([`docs/api.md`](docs/api.md)).

## Install

Download a binary from the [latest release](https://github.com/piekstra/target-cli/releases/latest),
or build from source:

```sh
make install                 # installs `tgt` onto your PATH from the reviewed Cargo.lock
```

The crate and repo are `target-cli`, and the binary is `tgt`. Later, `tgt self-update` fetches a
newer release.

## Usage

```console
$ tgt search "desk lamp" -n 2
1. Sample Lamp & Shade  [$1.00]
   brand: Sample Brand
   tcin:  00000001
   url:   https://www.target.com/p/sample-lamp/-/A-00000001

2. Sample Desk  [$2.00 - $3.00]
   tcin:  00000002
   url:   https://www.target.com/p/sample-desk/-/A-00000002

$ tgt get 00000001
Sample Lamp & Shade  [$1.00]
   brand: Sample Brand
   tcin:  00000001
   url:   https://www.target.com/p/sample-lamp/-/A-00000001
```

`--limit` (or `-n`) reads up to 200 results, 28 to a request. Prices and availability are per store,
so `--store <ID>` or `$TGT_STORE` names one. `tgt config set store <ID>` keeps it, and
`tgt config set limit <N>` keeps a limit.

If Target stops answering part way through a long read, `tgt search` prints what it read, says so on
stderr, and exits 5. Under `--json` the document carries the reason in `stopped_early`, as `tgt_search`
does, so a list is whole when the exit code is 0. A rate limit says how long to wait when Target does.

Should Target rotate the key built into `tgt`, `tgt config set key <KEY>`, `--key` or
`$TARGET_REDSKY_KEY` takes a newer one.

### JSON

`--json` prints one tagged document on stdout, and diagnostics go to stderr.

```console
$ tgt search "desk lamp" -n 5 --json | jq -r '.products[].url'
$ tgt get 00000001 --json
{
  "schema": "tgt-product/v1",
  "tcin": "00000001",
  "title": "Sample Lamp & Shade",
  "brand": "Sample Brand",
  "price": "$1.00",
  "url": "https://www.target.com/p/sample-lamp/-/A-00000001",
  "image": "https://target.scene7.com/is/image/Target/SAMPLE_1"
}
```

A price is Target's own text, and can be a range.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Anything else, such as a config file that cannot be read |
| 2 | Bad arguments, refused before any request |
| 4 | No product matched |
| 5 | Target failed, refused this client, rate limited it, or answered in a shape `tgt` does not recognize |

## Use with Claude

`tgt mcp` serves two read-only tools to Claude over MCP. They read what target.com shows anyone, with
no account. **No tool can buy anything, add to a cart or touch an account.**

| Tool | Answers |
|---|---|
| `tgt_search` | What Target lists for a keyword, with prices at a store |
| `tgt_product` | One product by its TCIN, with its price at a store |

Each takes an optional `store`, and uses `$TGT_STORE` or your `store` setting otherwise. Concise results
leave out the image link, and `response_format: detailed` adds it. A search that Target stops answering
part way returns what it read with `stopped_early` and a cursor.

**Claude Code.** Add the `target-cli` plugin from the
[piekstra plugin marketplace](https://github.com/piekstra/claude-plugins):

```console
$ claude plugin marketplace add piekstra/claude-plugins
$ claude plugin install target-cli@piekstra-plugins
```

or register the server yourself:

```console
$ claude mcp add tgt -- tgt mcp
```

**Claude Desktop.** Download `tgt-<version>.mcpb` from the
[latest release](https://github.com/piekstra/target-cli/releases/latest) and open it. It asks for
nothing and touches no keychain.

Any other MCP client that launches a local server can run `tgt mcp`.

Claude Desktop and launchd do not start from your shell, so they do not see an `XDG_CONFIG_HOME` set
in a shell profile and read `~/.config`.

**Claude on the web and mobile.** claude.ai reaches tools through a remote connector. `tgt`
reads your saved store from your computer and makes its requests from there, so it
serves the same tools from your computer to a connector you deploy into your own Cloudflare
account:

```console
$ tgt mcp --new-token            # once: a bearer token, saved in the keychain and printed for the Worker
$ tgt mcp --http --port 8774     # serve on 127.0.0.1 only
```

`--http` takes the token from `$TGT_MCP_TOKEN` if it is set, and otherwise from the keychain.
`tgt mcp --new-token --overwrite` replaces a saved one.
The connector, [cli-common's `connector/`](https://github.com/piekstra/cli-common/tree/main/connector#tools-from-your-computer-home-modules),
reaches that port through a Cloudflare Tunnel in your account; its README walks through the
tunnel, the VPC service, a launchd agent that keeps the server running, and the
`HOME_TGT_TOKEN` secret, with the threat model. `--http` binds 127.0.0.1 only, refuses to
start without the token, refuses any request whose `Host` is not loopback or that carries an
`Origin`, and serves only read-only tools, which is all `tgt` has. Tool results pass through
Cloudflare and reach Claude ([`SECURITY.md`](SECURITY.md)). While the computer sleeps, the
tools answer that it is offline.

After `tgt self-update`, restart the server: a running one keeps the old code, and its
launchd agent restarts it only when it exits. Its start line names the version.

```console
$ launchctl kickstart -k gui/$(id -u)/com.piekstra.tgt-mcp-http
```

`self-update` does not reach a Claude Desktop extension, which runs its own copy. Install
the new release's `.mcpb` too.

## Development

```sh
make verify     # fmt-check, lint, test, smoke
```

Tests are offline and use dummy fixtures. [`AGENTS.md`](AGENTS.md) has the layout and the rules.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option.
