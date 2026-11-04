# Contributing to target-cli

Thanks for helping out! This is a small, focused CLI over Target's public RedSky
API. It follows the family spec in
[cli-common](https://github.com/piekstra/cli-common)'s `DESIGN.md`, and [`AGENTS.md`](AGENTS.md) has this repo's rules.

## Before you start

- Open or comment on an issue describing the change.
- Branch from `main` (`feat/…`, `fix/…`, `docs/…`); don't push to `main` directly.
- Use [Conventional Commits](https://www.conventionalcommits.org/).

## Local checks (must pass)

```sh
make verify
```

If you touch RedSky parsing, add/adjust a unit test in `src/product.rs` with a
representative JSON node. Keep fields **best-effort** (only `tcin` is required) —
RedSky's schema drifts and we don't want a missing field to crash a lookup.

## Ground rules

- No secrets, tokens, or real account data in code, tests, fixtures, or commits.
  (This tool needs none — keep it that way.) CI runs `gitleaks`.
- Keep it personal-scale and within Target's Terms of Service. No features whose
  purpose is bulk scraping or hammering the API.
- Be kind — see [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Pull requests

Use the PR template and fill in the checklist. A maintainer (see
`.github/CODEOWNERS`) will review.

## License of contributions

By contributing, you agree your work is dual-licensed under **MIT OR Apache-2.0**.
