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
  `tgt` reports a 403 or 435 as a refusal and does not try to get around it.
- **`count` tops out at 28.** A longer read pages with `offset`.
- **A product answers 206**, partial content, so any 2xx counts.
- **The schema drifts.** Only `tcin` is relied on. Title, brand, price and links are best-effort.
- **A no-match search has not been seen on the wire.** `tgt` treats `products: []` as a real answer,
  and anything with no list as a changed API. If RedSky answers a no-match search with no list, that
  will read as an upstream error rather than "no products", which is the safer way to be wrong.
- **An unknown product has not been seen on the wire either.** A 404, or `data` with no product, is
  "not found". An answer with no `data` is a changed API.
- **`formatted_current_price` is display text**, and can be a range such as `$2.00 - $3.00`. It is
  passed through.
