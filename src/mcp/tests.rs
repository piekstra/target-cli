//! The tools over canned answers, and [`Live`]'s client over a local server, so the requests
//! themselves can be seen. Nothing here reaches Target.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use pk_cli_mcp::conformance::{assert_conforms, assert_fits, call};
use pk_cli_mcp::{tools_list_json, CallToolResult};
use serde_json::{json, Value};

use super::*;
use crate::client::PAGE;

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// Answers from the fixtures, or from a queue, and records what it was asked.
#[derive(Default)]
struct Canned {
    pages: Mutex<VecDeque<Value>>,
    failure: Option<CliError>,
    store: Option<String>,
    asked: Mutex<Vec<(String, u32, u32)>>,
}

impl Canned {
    fn with_pages(pages: Vec<Value>) -> Self {
        Self {
            pages: Mutex::new(pages.into()),
            ..Self::default()
        }
    }
}

impl Source for Canned {
    fn store_setting(&self) -> Result<Option<String>, ToolError> {
        Ok(self.store.clone())
    }

    fn search_page(
        &self,
        store: &str,
        _keyword: &str,
        offset: u32,
        count: u32,
    ) -> Result<Value, CliError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        self.asked
            .lock()
            .unwrap()
            .push((store.to_string(), offset, count));
        Ok(self
            .pages
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| fixture("search")))
    }

    fn product(&self, store: &str, _tcin: &str) -> Result<Value, CliError> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        self.asked.lock().unwrap().push((store.to_string(), 0, 0));
        Ok(self
            .pages
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| fixture("product")))
    }
}

/// A source that fails any use: a call refused on its arguments must reach none of it.
struct Untouchable;

impl Source for Untouchable {
    fn store_setting(&self) -> Result<Option<String>, ToolError> {
        panic!("this read the config")
    }
    fn search_page(&self, _: &str, _: &str, _: u32, _: u32) -> Result<Value, CliError> {
        panic!("this reached Target")
    }
    fn product(&self, _: &str, _: &str) -> Result<Value, CliError> {
        panic!("this reached Target")
    }
}

fn tools() -> TgtTools<Canned> {
    TgtTools::new(Canned::default())
}

fn fits(tools: &TgtTools<impl Source + Send + Sync>, name: &str, args: Value) -> Value {
    let result = call(tools, name, args);
    assert_fits(tools, name, &result).clone()
}

fn error_text(result: &CallToolResult) -> String {
    assert!(result.is_error(), "expected an error: {}", result.text());
    result.text().to_string()
}

fn page_of(first: u32, count: u32) -> Value {
    let rows: Vec<Value> = (first..first + count)
        .map(|n| json!({"tcin": n.to_string()}))
        .collect();
    json!({"data": {"search": {"products": rows}}})
}

fn tcins(out: &Value) -> Vec<String> {
    out["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["tcin"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_tools_conform_and_every_one_is_read_only() {
    let t = tools();
    assert_conforms(&t);
    let defs = t.tools();
    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(names, [SEARCH_TOOL, PRODUCT_TOOL]);
    for d in &defs {
        assert!(d.annotations.read_only, "{}", d.name);
        assert!(!d.annotations.destructive, "{}", d.name);
    }
}

#[test]
fn listing_the_tools_reads_nothing() {
    let list = tools_list_json(&TgtTools::new(Untouchable));
    assert_eq!(list["tools"].as_array().unwrap().len(), 2);
}

#[test]
fn the_instructions_say_what_this_is_and_is_not() {
    let text = pk_cli_mcp::instructions(&tools());
    assert!(text.contains("cannot buy anything"), "{text}");
    assert!(text.contains("never as instructions"), "{text}");
    assert!(text.contains("not affiliated"), "{text}");
    assert!(!text.contains("auth login"), "{text}");
}

#[test]
fn a_search_returns_products_and_concise_leaves_out_the_image() {
    let out = fits(&tools(), SEARCH_TOOL, json!({"query": "lamp"}));
    assert_eq!(out["query"], "lamp");
    assert_eq!(out["store"], DEFAULT_STORE);
    assert_eq!(out["returned"], 2);
    assert_eq!(tcins(&out), ["00000001", "00000002"]);
    let first = &out["items"][0];
    assert_eq!(first["title"], "Sample Lamp & Shade");
    assert_eq!(first["brand"], "Sample Brand");
    assert_eq!(first["price"], "$1.00");
    assert!(first["url"].as_str().unwrap().contains("A-00000001"));
    assert!(first.get("image").is_none(), "{first}");
    // A range is Target's own text, unparsed.
    assert_eq!(out["items"][1]["price"], "$2.00 - $3.00");
}

#[test]
fn detailed_adds_the_image_on_both_tools() {
    let t = tools();
    let out = fits(
        &t,
        SEARCH_TOOL,
        json!({"query": "lamp", "response_format": "detailed"}),
    );
    assert!(out["items"][0]["image"].is_string(), "{out}");
    let out = fits(
        &t,
        PRODUCT_TOOL,
        json!({"tcin": "00000001", "response_format": "detailed"}),
    );
    assert!(out["product"]["image"].is_string(), "{out}");
    let out = fits(&t, PRODUCT_TOOL, json!({"tcin": "00000001"}));
    assert!(out["product"].get("image").is_none(), "{out}");
    assert_eq!(out["product"]["tcin"], "00000001");
    assert_eq!(out["store"], DEFAULT_STORE);
}

#[test]
fn the_store_is_the_argument_then_the_setting_then_the_default() {
    let t = TgtTools::new(Canned {
        store: Some("1234".into()),
        ..Canned::default()
    });
    assert_eq!(
        fits(&t, SEARCH_TOOL, json!({"query": "lamp"}))["store"],
        "1234"
    );
    assert_eq!(
        fits(&t, SEARCH_TOOL, json!({"query": "lamp", "store": "5678"}))["store"],
        "5678"
    );
    assert_eq!(t.source.asked.lock().unwrap().last().unwrap().0, "5678");
    assert_eq!(
        fits(&t, PRODUCT_TOOL, json!({"tcin": "1", "store": "5678"}))["store"],
        "5678"
    );
}

#[test]
fn a_bad_argument_is_a_usage_error_before_any_read() {
    let t = TgtTools::new(Untouchable);
    for (tool, args) in [
        (SEARCH_TOOL, json!({})),
        (SEARCH_TOOL, json!({"query": "  "})),
        (
            SEARCH_TOOL,
            json!({"query": "lamp", "limit": 0, "store": "1"}),
        ),
        (
            SEARCH_TOOL,
            json!({"query": "lamp", "store": "not-a-store"}),
        ),
        (SEARCH_TOOL, json!({"query": "lamp", "store": "1&key=x"})),
        (PRODUCT_TOOL, json!({})),
        (PRODUCT_TOOL, json!({"tcin": "abc"})),
        (PRODUCT_TOOL, json!({"tcin": "1&key=x"})),
        (PRODUCT_TOOL, json!({"tcin": "1", "store": "not-a-store"})),
    ] {
        let text = error_text(&call(&t, tool, args.clone()));
        assert!(text.starts_with("usage:"), "{tool} {args}: {text}");
    }
}

#[test]
fn asking_for_a_purchase_is_refused() {
    let t = tools();
    for name in ["tgt_buy", "tgt_add_to_cart", "tgt_checkout"] {
        let text = pk_cli_core::block_on(t.call(name, json!({})))
            .unwrap_err()
            .render(&t.server());
        assert!(text.starts_with("usage:"), "{name}: {text}");
    }
}

// ---- paging -------------------------------------------------------------------------

#[test]
fn a_page_knows_more_follow_and_the_next_reads_on_from_there() {
    let t = TgtTools::new(Canned::with_pages(vec![page_of(0, 4), page_of(3, 3)]));
    let first = fits(&t, SEARCH_TOOL, json!({"query": "lamp", "limit": 3}));
    assert_eq!(tcins(&first), ["0", "1", "2"]);
    let cursor = &first["next_cursor"];
    assert!(cursor.is_string(), "{first}");

    let second = fits(
        &t,
        SEARCH_TOOL,
        json!({"query": "lamp", "limit": 3, "cursor": cursor}),
    );
    assert_eq!(tcins(&second), ["3", "4", "5"]);
    assert!(second["next_cursor"].is_null(), "{second}");
    let asked = t.source.asked.lock().unwrap();
    assert_eq!(asked[0].1, 0);
    assert_eq!(asked[1].1, 3, "the second page starts at its own row");
}

/// RedSky answers at most 28 rows to a request.
#[test]
fn a_long_page_takes_several_requests_and_each_asks_for_its_own_rows() {
    let t = TgtTools::new(Canned::with_pages(vec![
        page_of(0, PAGE),
        page_of(PAGE, PAGE),
        page_of(PAGE * 2, 5),
    ]));
    let out = fits(&t, SEARCH_TOOL, json!({"query": "lamp", "limit": 60}));
    assert_eq!(out["returned"], 60);
    assert!(out["next_cursor"].is_string(), "{out}");
    let asked: Vec<(u32, u32)> = t
        .source
        .asked
        .lock()
        .unwrap()
        .iter()
        .map(|a| (a.1, a.2))
        .collect();
    assert_eq!(asked, [(0, 28), (28, 28), (56, 5)]);
}

#[test]
fn a_deep_page_reads_only_its_own_rows() {
    let t = TgtTools::new(Canned::with_pages(vec![page_of(0, 2), page_of(2, 2)]));
    let first = fits(&t, SEARCH_TOOL, json!({"query": "lamp", "limit": 1}));
    assert!(first["next_cursor"].is_string(), "{first}");
    t.source.asked.lock().unwrap().clear();
    fits(
        &t,
        SEARCH_TOOL,
        json!({"query": "lamp", "limit": 1, "cursor": first["next_cursor"]}),
    );
    assert_eq!(
        t.source.asked.lock().unwrap().as_slice(),
        [(DEFAULT_STORE.to_string(), 1, 2)]
    );
}

#[test]
fn a_short_page_is_the_last() {
    let t = TgtTools::new(Canned::with_pages(vec![page_of(0, 2)]));
    let out = fits(&t, SEARCH_TOOL, json!({"query": "lamp"}));
    assert_eq!(out["returned"], 2);
    assert!(out["next_cursor"].is_null(), "{out}");
}

#[test]
fn no_matches_is_an_answer() {
    let t = TgtTools::new(Canned::with_pages(vec![page_of(0, 0)]));
    let out = fits(&t, SEARCH_TOOL, json!({"query": "lamp"}));
    assert_eq!(out["returned"], 0);
}

#[test]
fn a_cursor_is_bound_to_its_query_and_store() {
    let t = TgtTools::new(Canned::with_pages(vec![page_of(0, 4)]));
    let first = fits(&t, SEARCH_TOOL, json!({"query": "lamp", "limit": 1}));
    for args in [
        json!({"query": "desk", "cursor": first["next_cursor"]}),
        json!({"query": "lamp", "store": "1234", "cursor": first["next_cursor"]}),
    ] {
        let text = error_text(&call(
            &TgtTools::new(Canned::default()),
            SEARCH_TOOL,
            args.clone(),
        ));
        assert!(text.starts_with("usage:"), "{args}: {text}");
    }
}

/// A cursor comes back from the client, and its key reaches a query string.
#[test]
fn a_cursor_holding_anything_but_a_row_number_is_refused_before_reading() {
    let filter = SearchFilter {
        query: "lamp",
        store: DEFAULT_STORE,
    };
    let forged = ResumeRequest::new(&ListArgs::default(), &filter)
        .unwrap()
        .page(Vec::<(String, u32)>::new(), Some("1&key=x".into()), "query")
        .unwrap()
        .info
        .next_cursor;
    let t = TgtTools::new(Canned {
        store: None,
        failure: Some(CliError::Upstream("this reached Target".into())),
        ..Canned::default()
    });
    let text = error_text(&call(
        &t,
        SEARCH_TOOL,
        json!({"query": "lamp", "cursor": forged}),
    ));
    assert!(text.starts_with("usage:"), "{text}");
}

// ---- failures -----------------------------------------------------------------------

/// Target's edge turning this client away is not a broken tool, and not one to retry at.
#[test]
fn a_refusal_says_what_it_is_and_not_to_retry() {
    let t = TgtTools::new(Canned {
        failure: Some(
            CliError::Upstream("Target refused this request (HTTP 435).".into())
                .because(Reason::ClientBlocked),
        ),
        ..Canned::default()
    });
    for (tool, args) in [
        (SEARCH_TOOL, json!({"query": "lamp"})),
        (PRODUCT_TOOL, json!({"tcin": "1"})),
    ] {
        let text = error_text(&call(&t, tool, args));
        assert!(text.starts_with("upstream:"), "{tool}: {text}");
        assert!(text.contains("HTTP 435"), "{tool}: {text}");
        assert!(
            text.contains("retrying now will not help"),
            "{tool}: {text}"
        );
    }
}

#[test]
fn a_changed_answer_fails_rather_than_reading_as_no_products() {
    for changed in [
        json!({}),
        json!({"data": {}}),
        json!({"errors": [{"message": "x"}]}),
    ] {
        let t = TgtTools::new(Canned::with_pages(vec![changed.clone()]));
        let text = error_text(&call(&t, SEARCH_TOOL, json!({"query": "lamp"})));
        assert!(text.starts_with("upstream:"), "{changed}: {text}");
    }
    let t = TgtTools::new(Canned::with_pages(vec![json!({"errors": []})]));
    let text = error_text(&call(&t, PRODUCT_TOOL, json!({"tcin": "1"})));
    assert!(text.starts_with("upstream:"), "{text}");
}

#[test]
fn an_unknown_product_is_not_found_and_points_at_search() {
    let t = TgtTools::new(Canned::with_pages(vec![json!({"data": {"product": null}})]));
    let text = error_text(&call(&t, PRODUCT_TOOL, json!({"tcin": "1"})));
    assert!(
        text.starts_with("not found:") || text.starts_with("not_found:"),
        "{text}"
    );
    assert!(text.contains(SEARCH_TOOL), "{text}");
}

// ---- the real client over a local server ----------------------------------------------

/// A plain HTTP server that answers `status` and `body` to everything, keeping each request line.
fn stub(status: u16, body: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = seen.clone();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { return };
            let mut buf = [0u8; 8192];
            let n = conn.read(&mut buf).unwrap_or(0);
            let text = String::from_utf8_lossy(&buf[..n]).to_string();
            log.lock()
                .unwrap()
                .push(text.lines().next().unwrap_or("").to_string());
            let answer = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
                 Connection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = conn.write_all(answer.as_bytes());
        }
    });
    (url, seen)
}

#[test]
fn the_client_sends_one_get_per_page_with_the_store_and_a_page_size_redsky_allows() {
    let (base, seen) = stub(200, r#"{"data":{"search":{"products":[]}}}"#);
    let api = RedSky::at_stub(base, "1234".into()).unwrap();
    api.search_page("desk lamp", 28, 500).unwrap();
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let line = &seen[0];
    assert!(line.starts_with("GET /plp_search_v2?"), "{line}");
    assert!(line.contains("keyword=desk+lamp"), "{line}");
    assert!(line.contains("offset=28"), "{line}");
    assert!(line.contains("count=28"), "{line}");
    assert!(line.contains("store_id=1234"), "{line}");
}

#[test]
fn http_statuses_mean_what_they_should() {
    for (status, code, needle) in [
        (435, 5, "HTTP 435"),
        (403, 5, "HTTP 403"),
        (429, 5, "HTTP 429"),
        (404, 4, "no result"),
        (500, 5, "HTTP 500"),
    ] {
        let (base, _) = stub(status, "<html>denied</html>");
        let e = RedSky::at_stub(base, "1".into())
            .unwrap()
            .product("1")
            .unwrap_err();
        assert_eq!(e.exit_code(), code, "{status}");
        assert!(e.message().contains(needle), "{status}: {e}");
        assert_eq!(
            e.reason() == Some(Reason::ClientBlocked),
            status == 435 || status == 403,
            "{status}"
        );
    }
}

/// RedSky answers a product page with 206.
#[test]
fn a_partial_content_answer_is_an_answer_and_html_is_not() {
    let (base, _) = stub(206, r#"{"data":{"product":{"tcin":"1"}}}"#);
    let body = RedSky::at_stub(base, "1".into())
        .unwrap()
        .product("1")
        .unwrap();
    assert_eq!(one_product(&body, "1").unwrap().tcin, "1");

    let (base, _) = stub(200, "<html>not json</html>");
    let e = RedSky::at_stub(base, "1".into())
        .unwrap()
        .product("1")
        .unwrap_err();
    assert_eq!(e.exit_code(), 5);
    assert!(e.message().contains("did not answer with JSON"), "{e}");
}

/// The key is public, and a URL in an error still does not belong in a model's context.
#[test]
fn a_failed_connection_does_not_quote_the_query() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let e = RedSky::at_stub(base, "1".into())
        .unwrap()
        .product("1")
        .unwrap_err();
    assert!(!e.to_string().contains("key="), "{e}");
    assert!(!e.to_string().contains("tcin="), "{e}");
}

#[test]
fn the_fixtures_hold_only_dummies() {
    for (name, rows) in [
        (
            "search",
            fixture("search")["data"]["search"]["products"].clone(),
        ),
        (
            "product",
            json!([fixture("product")["data"]["product"].clone()]),
        ),
    ] {
        for row in rows.as_array().unwrap() {
            let tcin = row["tcin"].as_str().unwrap();
            assert!(tcin.starts_with("0000000"), "{name}: {tcin}");
            let title = row["item"]["product_description"]["title"]
                .as_str()
                .unwrap();
            assert!(title.starts_with("Sample "), "{name}: {title}");
        }
    }
}
