# Fixtures

`search.json` and `product.json` are RedSky answers reduced to the fields `src/product.rs` reads.
Keys and nesting follow what that parser has read since v0.1.0. Every value is a dummy: `SAMPLE`
titles and brands, TCINs of `0000000n`, single-digit prices.

They are **not fresh captures**. On 2026-11-03 RedSky answered HTTP 435 to every request from the
machine this was built on, so its live shape could not be checked (`docs/api.md`).
