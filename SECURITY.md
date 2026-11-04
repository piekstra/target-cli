# Security policy

## Reporting a vulnerability

Please report security issues **privately** — do not open a public issue.

- Preferred: open a [GitHub private security advisory](https://docs.github.com/en/code-security/security-advisories)
  on this repo ("Security" tab → "Report a vulnerability").
- Or contact the maintainer (see `.github/CODEOWNERS`).

We aim to acknowledge within a few days and coordinate a fix/disclosure with you.

## Notes for this tool

`tgt` uses **no credential for Target**. It sends anonymous read-only requests to RedSky, with the
key target.com's own client-side code carries. That key is public, so it is allowlisted in
`.gitleaks.toml`, and `--key` may pass it on the command line. It reads nothing a visitor to the site
cannot.

Your search terms go to Target. There is no telemetry, and no other third party is involved unless you
set one up: `tgt mcp` results reach Claude, and `--http` results pass through your own Cloudflare
account.

The only secret this tool can hold is the bearer token of `mcp --http`, below. No other state is kept
beyond the config file, which holds a store id and a limit. Dependency advisories are the other concern
here, and `cargo audit` and `cargo deny` run in CI.

## MCP tools

`tgt mcp` serves two read-only tools to Claude: `tgt_search` and `tgt_product`. None writes anything,
and none takes a path or a URL. **No tool can buy anything, add to a cart or touch an account**, and
`RedSky` has no method that could. A TCIN, a store id and a cursor's key are checked to be digits
before they reach a query string. The config is read on each call. On stdio nothing touches the
keychain, and listing the tools reads no file.

**What reaches Claude.** Tool results go to Claude as part of your conversation: product listings and
prices, for the store you name. Titles and brands are written by sellers, so they are third-party data,
and the server's instructions tell the model to treat them as data, never as instructions.

`tgt mcp --http --port <PORT>` serves the same tools for your own Cloudflare Worker, which
reaches it through your own Cloudflare Tunnel (cli-common's `connector/`). It binds 127.0.0.1
only, with no flag for another address, and refuses to start without a bearer token of 32 or
more characters, from `TGT_MCP_TOKEN` or the keychain item `mcp-http-token` (`piekstra.tgt`),
which `tgt mcp --new-token` makes from 256 random bits and replaces only with `--overwrite`.
Every request must carry the token (compared in constant time) and a `Host` of `127.0.0.1` or
`localhost`, and must not carry an `Origin`; there are no CORS headers. A copy of that token
lives in your Worker as its `HOME_TGT_TOKEN` secret. The data is public, so a leaked token
gives no account to anyone: it can read Target's listings from your computer's address.

**Tool results also cross Cloudflare on this path**, through your tunnel and your Worker, on
their way to Claude. Who holds what, and what a leaked token can do, is in the connector's
[threat model](https://github.com/piekstra/cli-common/blob/main/connector/README.md#threat-model).

`tgt` has no `auth logout`, since it has no sign-in. To revoke the connector's access, run
`tgt mcp --new-token --overwrite` and restart the server (`launchctl kickstart -k gui/$(id
-u)/com.piekstra.tgt-mcp-http`); the old token works until the restart, and a token in
`$TGT_MCP_TOKEN` wins until you change it. To stop serving, run `launchctl bootout gui/$(id
-u)/com.piekstra.tgt-mcp-http` and stop the tunnel. To remove the token on macOS, run
`security delete-generic-password -s piekstra.tgt -a mcp-http-token`.

## Supported versions

Pre-1.0: only the latest release receives fixes.
