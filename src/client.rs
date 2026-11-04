//! Thin, polite client over RedSky, the aggregation API target.com's own front end calls. It has
//! no published contract. See `docs/api.md`.

use std::time::Duration;

use pk_cli_core::error::Reason;
use pk_cli_core::CliError;
use serde_json::Value;

/// The key target.com's own client-side code carries. It is not a secret and not a credential:
/// it reads nothing a visitor to the site cannot. `--key`, `$TARGET_REDSKY_KEY` and the `key`
/// setting override it, should it rotate.
pub const DEFAULT_KEY: &str = "9f36aeafbe60771e321a7cc95a78140772ab3e96";

/// Prices and availability are per store, so a request has to name one.
pub const DEFAULT_STORE: &str = "3991";

/// The furthest one search reads in one go, at [`PAGE`] rows a request.
pub const MAX_LIMIT: u32 = 200;

/// RedSky answers at most this many products to a search request.
pub const PAGE: u32 = 28;

const BASE: &str = "https://redsky.target.com/redsky_aggregations/v1/web";
const VISITOR_ID: &str = "0193A1B2C3D4E5F6";
const UA: &str = concat!(
    "target-cli/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/piekstra/target-cli)"
);
/// How long one request may take.
pub const TIMEOUT: Duration = Duration::from_secs(20);

/// Target answering that it will not serve this client. A rotated key could answer this way too.
const REFUSED: [u16; 3] = [401, 403, 435];

/// What to do about a refusal, for a person at a terminal and for a model to pass on.
pub const REFUSAL_REMEDY: &str =
    "it may pass, and retrying now will not help. If it goes on, Target may have rotated \
     the key built into tgt: `tgt config set key <KEY>`, `--key` or `$TARGET_REDSKY_KEY` takes a \
     newer one, and a newer release of tgt may carry it";

/// A client bound to one store.
pub struct RedSky {
    client: reqwest::blocking::Client,
    key: String,
    store: String,
    base: String,
}

impl RedSky {
    pub fn new(key: Option<String>, store: String) -> Result<Self, CliError> {
        Self::at(BASE.to_string(), key, store)
    }

    /// A client for a local stub, for tests of what is sent.
    #[cfg(test)]
    pub fn at_stub(base: String, store: String) -> Result<Self, CliError> {
        Self::at(base, None, store)
    }

    fn at(base: String, key: Option<String>, store: String) -> Result<Self, CliError> {
        let key = key
            .filter(|k| !k.is_empty())
            .unwrap_or_else(|| DEFAULT_KEY.to_string());
        let client = pk_cli_http::builder("tgt", env!("CARGO_PKG_VERSION"))
            .user_agent(UA)
            .timeout(TIMEOUT)
            .redirect(pk_cli_core::transport::secure_redirects())
            .build()
            .map_err(|e| CliError::Other(format!("failed to build HTTP client: {e}")))?;
        Ok(Self {
            client,
            key,
            store,
            base,
        })
    }

    fn get_json(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Value, CliError> {
        let url = format!("{}/{endpoint}", self.base);
        let resp = self.client.get(&url).query(params).send()?;
        let status = resp.status().as_u16();
        if REFUSED.contains(&status) {
            return Err(CliError::Upstream(format!(
                "Target refused this request (HTTP {status}). Its site turns away some automated \
                 clients, and this tool does not try to get around that"
            ))
            .because(Reason::ClientBlocked)
            .with_remedy(REFUSAL_REMEDY));
        }
        match status {
            429 => {
                let wait = match pk_cli_http::retry_after(resp.headers()) {
                    Some(wait) => format!("wait {} seconds before trying again", wait.as_secs()),
                    None => "wait a few minutes before trying again, since retrying at once \
                             keeps the limit in force"
                        .to_string(),
                };
                return Err(
                    CliError::Upstream("rate limited by Target (HTTP 429)".into())
                        .with_remedy(wait),
                );
            }
            404 => return Err(CliError::NotFound(format!("no result from {endpoint}"))),
            // A product page commonly answers 206, partial content.
            200..=299 => {}
            _ => return Err(CliError::Upstream(format!("HTTP {status} from {endpoint}"))),
        }
        resp.json::<Value>()
            .map_err(|_| CliError::Upstream(format!("{endpoint} did not answer with JSON")))
    }

    /// One whole page of a keyword search, from row `offset`. The rows are at
    /// `data.search.products`.
    pub fn search_page(&self, keyword: &str, offset: u32) -> Result<Value, CliError> {
        let count = PAGE.to_string();
        let offset = offset.to_string();
        let page = format!("/s/{keyword}");
        let params = [
            ("key", self.key.as_str()),
            ("keyword", keyword),
            ("channel", "WEB"),
            ("count", count.as_str()),
            ("offset", offset.as_str()),
            ("page", page.as_str()),
            ("platform", "desktop"),
            ("pricing_store_id", self.store.as_str()),
            ("store_id", self.store.as_str()),
            ("visitor_id", VISITOR_ID),
        ];
        self.get_json("plp_search_v2", &params)
    }

    /// One product by TCIN. The product is at `data.product`.
    pub fn product(&self, tcin: &str) -> Result<Value, CliError> {
        let page = format!("/p/A-{tcin}");
        let params = [
            ("key", self.key.as_str()),
            ("tcin", tcin),
            ("channel", "WEB"),
            ("page", page.as_str()),
            ("pricing_store_id", self.store.as_str()),
            ("store_id", self.store.as_str()),
            ("visitor_id", VISITOR_ID),
        ];
        self.get_json("pdp_client_v1", &params)
    }
}
