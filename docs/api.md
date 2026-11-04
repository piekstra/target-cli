# RedSky, as `tgt` uses it

RedSky is the aggregation API target.com's own front end calls. It has no published contract, so
everything here is what was observed, and it can change without notice.

## Requests

Base: `https://redsky.target.com/redsky_aggregations/v1/web`. Both are `GET`.

| Endpoint | Used by | Answer |
|---|---|---|
| `plp_search_v2` | `tgt search`, `tgt_search` | `data.search.products[]` |
| `pdp_client_v1` | `tgt get`, `tgt_product` | `data.product` |

Every request sends `key`, `channel=WEB`, `pricing_store_id`, `store_id`, `visitor_id` and `page`.
A search adds `keyword`, `count`, `offset` and `platform=desktop`; a product adds `tcin`.

`key` is the key target.com's own client-side code carries, so it is public. `visitor_id` is a fixed
placeholder.

## Traps

- **A bot wall sits in front of it.** Target answers HTTP 435 with an HTML page reading "Access to
  this page has been denied" when it declines a client. On 2026-11-03 it answered that to every
  request from the machine this was built on, whatever the endpoint or the key. So nothing since
  v0.1.0 has been checked against a live answer, including the shapes below and `tests/fixtures/`.
  `tgt` reports a 401, 403 or 435 as a refusal and does not try to get around it.
- **The key could rotate.** How RedSky rejects an old key has not been seen, so a refusal also names
  the overrides: `tgt config set key <KEY>`, `--key` and `$TARGET_REDSKY_KEY`. The setting is the one
  that reaches the Claude Desktop extension, which starts with no environment of the user's.
- **`count` tops out at 28.** A longer read pages with `offset`, with a quarter second between
  requests.
- **Offset paging is unverified.** v0.1.0 only ever sent `offset=0`, and no request since has been
  answered. Three assumptions rest on that: `offset` moves forward, a page shorter than `count` ends
  the list, and a 404 past row 0 ends it too. If RedSky caps `count` lower or filters rows out of a
  page, a short page would read as the end of the list. A page with nothing new fails as a changed
  API, which catches an ignored `offset` when one call spans pages. A page that repeats a row or two
  among new ones is taken as the list moving, and the repeats are dropped. A call that fits in one
  request cannot see any of that.
- **A failure part way keeps what was read.** `tgt_search` answers a short page with `stopped_early`
  and a cursor. `tgt search` prints the rows, says so on stderr and in `stopped`, and exits with the
  failure's code. `tgt_search` also stops asking at 90 seconds, inside the MCP server's 120-second
  call limit.
- **A 429 can carry `Retry-After`,** which `tgt` passes on as how long to wait.
- **A product answers 206**, partial content, so any 2xx counts.
- **The schema drifts.** Only `tcin` is relied on. Title, brand, price and links are best-effort.
- **A no-match search has not been seen on the wire.** `tgt` treats `products: []` as a real answer,
  and anything with no list as a changed API. If RedSky answers a no-match search with no list, that
  will read as an upstream error rather than "no products", which is the safer way to be wrong.
- **An unknown product has not been seen on the wire either.** A 404, or `data` with no product, is
  "not found". An answer with no `data` is a changed API.
- **`formatted_current_price` is display text**, and can be a range such as `$2.00 - $3.00`. It is
  passed through.
