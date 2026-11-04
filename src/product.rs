//! The product model, and what RedSky's answers mean. RedSky's schema is unpublished and drifts,
//! so every field beyond `tcin` is best-effort, but an answer with no list at all fails rather
//! than reading as "no products".

use std::collections::HashSet;
use std::time::{Duration, Instant};

use pk_cli_core::CliError;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::client::PAGE;

/// The gap between one search's requests. The service sits behind a bot wall, and a long read is
/// several requests in a row.
const PAGE_PAUSE: Duration = Duration::from_millis(250);

/// A normalized Target product, stable across search and product responses.
#[derive(Serialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Product {
    pub tcin: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// As Target formats it, which can be a range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Canonical product URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Read a nested string field, unescaping common HTML entities.
fn s(v: &Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for k in path {
        cur = cur.get(k)?;
    }
    cur.as_str().map(unescape)
}

/// Minimal HTML-entity unescape for product titles. `&amp;` goes last, so `&amp;lt;` stays `&lt;`.
fn unescape(s: &str) -> String {
    s.replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

impl Product {
    /// Parse a `data.search.products[]` element or the `data.product` object.
    pub fn from_node(v: &Value) -> Option<Product> {
        let tcin = v.get("tcin")?.as_str()?.to_string();
        let title = s(v, &["item", "product_description", "title"]).unwrap_or_default();
        let brand = s(v, &["item", "primary_brand", "name"]);
        let url = s(v, &["item", "enrichment", "buy_url"]);
        let price = s(v, &["price", "formatted_current_price"]);
        let image = s(
            v,
            &["item", "enrichment", "image_info", "primary_image", "url"],
        )
        .or_else(|| s(v, &["item", "enrichment", "images", "primary_image_url"]));
        Some(Product {
            tcin,
            title,
            brand,
            price,
            url,
            image,
        })
    }
}

fn changed(what: &str) -> CliError {
    CliError::Upstream(format!(
        "Target's {what} answer is not in a shape this tool recognizes"
    ))
}

/// The rows of one search page. A present, empty list is a real answer. Anything else is a
/// changed API, and rows with no TCIN among them are not products at all.
pub fn search_rows(body: &Value) -> Result<&[Value], CliError> {
    let rows = body
        .pointer("/data/search/products")
        .and_then(Value::as_array)
        .ok_or_else(|| changed("search"))?;
    if !rows.is_empty() && rows.iter().all(|r| Product::from_node(r).is_none()) {
        return Err(changed("search"));
    }
    Ok(rows)
}

/// The one product in a product answer.
pub fn one_product(body: &Value, tcin: &str) -> Result<Product, CliError> {
    let data = body
        .get("data")
        .filter(|d| d.is_object())
        .ok_or_else(|| changed("product"))?;
    match data.get("product") {
        None | Some(Value::Null) => Err(CliError::NotFound(format!("no product with tcin {tcin}"))),
        Some(node) => Product::from_node(node).ok_or_else(|| changed("product")),
    }
}

/// Why a [`walk`] ended short of what it wanted, with more of the list possibly unread.
#[derive(Debug)]
pub enum Stopped {
    /// A page after the first failed.
    Failed(CliError),
    /// The deadline passed before the next page.
    OutOfTime,
}

/// What a [`walk`] read.
#[derive(Debug)]
pub struct Walk {
    /// Each product, with the row to read on from after it.
    pub found: Vec<(u32, Product)>,
    /// The row a later read starts from.
    pub at: u32,
    pub stopped: Option<Stopped>,
}

/// Read up to `wanted` products from row `offset`, [`PAGE`] at a time. `page` is one request for
/// `(offset, count)`. `deadline`, if any, stops it asking for another page.
///
/// A failure once rows are in hand keeps them, and says so in [`Walk::stopped`]. A short page ends
/// the list, and past row 0 so does a 404, both assumptions recorded in `docs/api.md`. A list that
/// moves between requests can repeat a row, which is dropped. A page with nothing new is RedSky
/// not honouring `offset`, which would otherwise repeat the same rows forever, so that fails.
pub fn walk(
    mut page: impl FnMut(u32, u32) -> Result<Value, CliError>,
    offset: u32,
    wanted: u32,
    deadline: Option<Instant>,
) -> Result<Walk, CliError> {
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut at = offset;
    let mut stopped = None;
    let mut first = true;
    while (found.len() as u32) < wanted {
        if !first {
            if deadline.is_some_and(|d| Instant::now() >= d) {
                stopped = Some(Stopped::OutOfTime);
                break;
            }
            std::thread::sleep(PAGE_PAUSE);
        }
        first = false;
        let ask = (wanted - found.len() as u32).min(PAGE);
        let body = page(at, ask);
        let rows = match body.as_ref().map_err(CliError::clone).and_then(search_rows) {
            Ok(rows) => rows,
            Err(CliError::NotFound(_)) if at > 0 => break,
            Err(e) if found.is_empty() => return Err(e),
            Err(e) => {
                stopped = Some(Stopped::Failed(e));
                break;
            }
        };
        let before = found.len();
        for (index, row) in rows.iter().enumerate() {
            if let Some(product) = Product::from_node(row) {
                if seen.insert(product.tcin.clone()) {
                    found.push((at + index as u32 + 1, product));
                }
            }
        }
        if !rows.is_empty() && found.len() == before {
            return Err(changed("search"));
        }
        at += rows.len() as u32;
        if (rows.len() as u32) < ask {
            break;
        }
    }
    found.truncate(wanted as usize);
    Ok(Walk { found, at, stopped })
}

/// A TCIN is digits, and reaches a query string.
pub fn checked_tcin(raw: &str) -> Result<String, CliError> {
    digits(raw, 20)
        .ok_or_else(|| CliError::Usage(format!("a TCIN is a number such as 12345678, got `{raw}`")))
}

/// A store id is digits, and reaches a query string.
pub fn checked_store(raw: &str) -> Result<String, CliError> {
    digits(raw, 8)
        .ok_or_else(|| CliError::Usage(format!("a store id is a number such as 1234, got `{raw}`")))
}

fn digits(raw: &str, most: usize) -> Option<String> {
    let raw = raw.trim();
    let ok = !raw.is_empty() && raw.len() <= most && raw.chars().all(|c| c.is_ascii_digit());
    ok.then(|| raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(tcin: u32) -> Value {
        json!({"tcin": tcin.to_string()})
    }

    fn page_of(first: u32, count: u32) -> Value {
        let rows: Vec<Value> = (first..first + count).map(node).collect();
        json!({"data": {"search": {"products": rows}}})
    }

    #[test]
    fn parses_a_node_and_unescapes_its_title() {
        let node = json!({
            "tcin": "00000001",
            "item": {
                "product_description": {"title": "Sample Lamp &amp; Shade"},
                "primary_brand": {"name": "Sample Brand"},
                "enrichment": {"buy_url": "https://www.target.com/p/sample/-/A-00000001"}
            },
            "price": {"formatted_current_price": "$1.00"}
        });
        let p = Product::from_node(&node).expect("should parse");
        assert_eq!(p.tcin, "00000001");
        assert_eq!(p.title, "Sample Lamp & Shade");
        assert_eq!(p.brand.as_deref(), Some("Sample Brand"));
        assert_eq!(p.price.as_deref(), Some("$1.00"));
        assert!(p.url.unwrap().contains("A-00000001"));
    }

    #[test]
    fn an_escaped_entity_unescapes_only_once() {
        assert_eq!(unescape("a &amp;lt; b"), "a &lt; b");
    }

    #[test]
    fn missing_tcin_yields_none() {
        assert!(Product::from_node(&json!({"item": {}})).is_none());
    }

    #[test]
    fn tolerates_missing_optional_fields() {
        let p = Product::from_node(&json!({"tcin": "1"})).expect("tcin only is enough");
        assert_eq!(p.title, "");
        assert!(p.brand.is_none());
        assert!(p.url.is_none());
    }

    #[test]
    fn an_empty_list_is_an_answer_and_a_missing_one_is_not() {
        assert!(search_rows(&page_of(0, 0)).unwrap().is_empty());
        for changed in [
            json!({}),
            json!({"data": {}}),
            json!({"data": {"search": {}}}),
            json!({"data": {"search": {"products": "none"}}}),
            json!({"data": {"search": {"products": [{"no": "tcin"}]}}}),
            json!({"errors": [{"message": "an example"}]}),
        ] {
            assert_eq!(
                search_rows(&changed).unwrap_err().exit_code(),
                5,
                "{changed}"
            );
        }
    }

    #[test]
    fn a_product_answer_says_found_not_found_or_changed() {
        let found = json!({"data": {"product": {"tcin": "1"}}});
        assert_eq!(one_product(&found, "1").unwrap().tcin, "1");
        for missing in [json!({"data": {}}), json!({"data": {"product": null}})] {
            assert_eq!(
                one_product(&missing, "1").unwrap_err().exit_code(),
                4,
                "{missing}"
            );
        }
        for changed in [
            json!({}),
            json!({"data": null}),
            json!({"data": {"product": {"no": "tcin"}}}),
        ] {
            assert_eq!(
                one_product(&changed, "1").unwrap_err().exit_code(),
                5,
                "{changed}"
            );
        }
    }

    /// A page's rows, or the failure in its place, in order, with what was asked.
    fn walk_over(
        answers: Vec<Result<Value, CliError>>,
        offset: u32,
        wanted: u32,
    ) -> (Result<Walk, CliError>, Vec<(u32, u32)>) {
        let mut answers = answers.into_iter();
        let mut asked = Vec::new();
        let result = walk(
            |at, count| {
                asked.push((at, count));
                answers.next().expect("asked for a page too many")
            },
            offset,
            wanted,
            None,
        );
        (result, asked)
    }

    fn tcins(walked: &Walk) -> Vec<&str> {
        walked.found.iter().map(|(_, p)| p.tcin.as_str()).collect()
    }

    #[test]
    fn a_walk_asks_for_a_page_at_a_time_and_stops_at_a_short_one() {
        let (walked, asked) =
            walk_over(vec![Ok(page_of(1, PAGE)), Ok(page_of(1 + PAGE, 5))], 0, 100);
        let walked = walked.unwrap();
        assert_eq!(asked, [(0, 28), (28, 28)]);
        assert_eq!(walked.found.len(), 33);
        assert_eq!(walked.found.last().unwrap().0, 33);
        assert!(walked.stopped.is_none());
    }

    #[test]
    fn a_walk_asks_for_no_more_than_it_wants_and_starts_where_it_is_told() {
        let (walked, asked) =
            walk_over(vec![Ok(page_of(1, PAGE)), Ok(page_of(1 + PAGE, 2))], 5, 30);
        assert_eq!(asked, [(5, 28), (33, 2)]);
        assert_eq!(walked.unwrap().found.len(), 30);
    }

    #[test]
    fn a_skipped_row_does_not_move_the_next_rows_place() {
        let body = json!({"data": {"search": {"products": [
            {"tcin": "1"}, {"no": "tcin"}, {"tcin": "3"},
        ]}}});
        let (walked, _) = walk_over(vec![Ok(body)], 10, 28);
        let keys: Vec<u32> = walked.unwrap().found.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, [11, 13]);
    }

    #[test]
    fn a_failure_on_the_first_page_is_the_answer() {
        let (walked, _) = walk_over(vec![Err(CliError::Upstream("HTTP 500".into()))], 0, 10);
        assert_eq!(walked.unwrap_err().exit_code(), 5);
    }

    #[test]
    fn a_failure_on_a_later_page_keeps_what_was_read_and_where_to_read_on() {
        let (walked, _) = walk_over(
            vec![
                Ok(page_of(1, PAGE)),
                Err(CliError::Upstream("HTTP 500".into())),
            ],
            0,
            100,
        );
        let walked = walked.unwrap();
        assert_eq!(walked.found.len(), PAGE as usize);
        assert_eq!(walked.at, PAGE);
        assert!(matches!(walked.stopped, Some(Stopped::Failed(_))));
    }

    #[test]
    fn a_changed_answer_on_a_later_page_keeps_what_was_read_too() {
        let (walked, _) = walk_over(vec![Ok(page_of(1, PAGE)), Ok(json!({}))], 0, 100);
        assert!(matches!(walked.unwrap().stopped, Some(Stopped::Failed(_))));
    }

    /// A 404 at the start is no result at all. Past it, the list has ended.
    #[test]
    fn a_404_past_the_first_row_ends_the_list_and_at_the_first_row_does_not() {
        let gone = || Err(CliError::NotFound("no result from plp_search_v2".into()));
        let (walked, _) = walk_over(vec![Ok(page_of(1, PAGE)), gone()], 0, 100);
        let walked = walked.unwrap();
        assert_eq!(walked.found.len(), PAGE as usize);
        assert!(walked.stopped.is_none());

        let (walked, _) = walk_over(vec![gone()], 28, 10);
        assert!(walked.unwrap().found.is_empty());

        let (walked, _) = walk_over(vec![gone()], 0, 10);
        assert_eq!(walked.unwrap_err().exit_code(), 4);
    }

    #[test]
    fn a_passed_deadline_stops_before_the_next_page_and_never_before_the_first() {
        let mut asked = 0;
        let walked = walk(
            |_, _| {
                asked += 1;
                Ok(page_of(1, PAGE))
            },
            0,
            100,
            Some(Instant::now()),
        )
        .unwrap();
        assert_eq!(asked, 1);
        assert_eq!(walked.at, PAGE);
        assert!(matches!(walked.stopped, Some(Stopped::OutOfTime)));
    }

    #[test]
    fn a_page_with_nothing_new_is_a_changed_api_and_not_more_products() {
        let (walked, _) = walk_over(vec![Ok(page_of(1, PAGE)), Ok(page_of(1, PAGE))], 0, 100);
        assert_eq!(walked.unwrap_err().exit_code(), 5);
    }

    /// A list that moved between requests pushes a row onto the next page.
    #[test]
    fn a_repeated_row_is_dropped_and_the_rest_keep_their_places() {
        let (walked, _) = walk_over(vec![Ok(page_of(1, PAGE)), Ok(page_of(PAGE, 3))], 0, 100);
        let walked = walked.unwrap();
        assert_eq!(walked.found.len(), PAGE as usize + 2);
        assert_eq!(tcins(&walked).last().unwrap(), &(PAGE + 2).to_string());
        assert_eq!(walked.found.last().unwrap().0, PAGE + 3);
    }

    #[test]
    fn rows_beyond_what_was_asked_for_are_dropped() {
        let (walked, _) = walk_over(vec![Ok(page_of(1, 5))], 0, 2);
        assert_eq!(walked.unwrap().found.len(), 2);
    }

    #[test]
    fn ids_are_digits_of_a_sane_length() {
        assert_eq!(checked_tcin(" 123 ").unwrap(), "123");
        assert_eq!(checked_store("3991").unwrap(), "3991");
        for bad in ["", " ", "12a", "1&x=2", "../1", "123456789012345678901"] {
            assert_eq!(checked_tcin(bad).unwrap_err().exit_code(), 2, "{bad}");
        }
        for bad in ["", "abc", "1 2", "123456789"] {
            assert_eq!(checked_store(bad).unwrap_err().exit_code(), 2, "{bad}");
        }
    }
}
