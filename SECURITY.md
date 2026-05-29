# Security policy

## Reporting a vulnerability

Please report security issues **privately** — do not open a public issue.

- Preferred: open a [GitHub private security advisory](https://docs.github.com/en/code-security/security-advisories)
  on this repo ("Security" tab → "Report a vulnerability").
- Or contact the maintainer (see `.github/CODEOWNERS`).

We aim to acknowledge within a few days and coordinate a fix/disclosure with you.

## Notes for this tool

`target-cli` uses **no credentials** and stores **no state** — it makes anonymous
read-only requests to Target's public RedSky API. The most relevant concerns are
therefore dependency advisories (`cargo audit` / `cargo deny` run in CI) and not
introducing any secret-handling by accident. The broader ecosystem security model
lives in
[baby-registry/docs/security-model.md](https://github.com/piekstra/baby-registry/blob/main/docs/security-model.md).

## Supported versions

Pre-1.0: only the latest release receives fixes.
