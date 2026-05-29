use std::time::Duration;

use serde_json::Value;

use crate::error::AppError;

/// Built-in default RedSky public key (the shared token baked into target.com's
/// own front end). Overridable via `--key` / `$TARGET_REDSKY_KEY` if it rotates.
pub const DEFAULT_KEY: &str = "9f36aeafbe60771e321a7cc95a78140772ab3e96";

const BASE: &str = "https://redsky.target.com/redsky_aggregations/v1/web";
const VISITOR_ID: &str = "0193A1B2C3D4E5F6";
const UA: &str = concat!(
    "target-cli/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/piekstra/target-cli)"
);

/// Thin, polite client over Target's public RedSky aggregation API.
pub struct RedSky {
    client: reqwest::blocking::Client,
    key: String,
    store: String,
}

impl RedSky {
    pub fn new(key: String, store: String) -> Result<Self, AppError> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(UA)
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| AppError::Other(format!("failed to build HTTP client: {e}")))?;
        Ok(Self { client, key, store })
    }

    fn get_json(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Value, AppError> {
        let url = format!("{BASE}/{endpoint}");
        let resp = self.client.get(&url).query(params).send()?;
        let status = resp.status();
        if status.as_u16() == 429 {
            return Err(AppError::RateLimited);
        }
        if status.as_u16() == 404 {
            return Err(AppError::NotFound(format!("no result from {endpoint}")));
        }
        // RedSky PDP commonly returns 206 (partial content); is_success covers it.
        if !status.is_success() {
            return Err(AppError::Network(format!(
                "HTTP {} from {endpoint}",
                status.as_u16()
            )));
        }
        Ok(resp.json::<Value>()?)
    }

    /// Keyword search → `data.search.products[]`.
    pub fn search(&self, keyword: &str, count: u32) -> Result<Value, AppError> {
        let count_s = count.clamp(1, 28).to_string();
        let page = format!("/s/{keyword}");
        let params = [
            ("key", self.key.as_str()),
            ("keyword", keyword),
            ("channel", "WEB"),
            ("count", count_s.as_str()),
            ("offset", "0"),
            ("page", page.as_str()),
            ("platform", "desktop"),
            ("pricing_store_id", self.store.as_str()),
            ("store_id", self.store.as_str()),
            ("visitor_id", VISITOR_ID),
        ];
        self.get_json("plp_search_v2", &params)
    }

    /// Product detail by TCIN → `data.product`.
    pub fn product(&self, tcin: &str) -> Result<Value, AppError> {
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
