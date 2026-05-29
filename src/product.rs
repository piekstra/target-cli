use serde::Serialize;
use serde_json::Value;

/// A normalized Target product, stable across search and PDP responses.
#[derive(Serialize, Debug, PartialEq)]
pub struct Product {
    pub tcin: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Canonical product URL — feed this straight into `babylist-cli add`.
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

/// Minimal HTML-entity unescape for product titles.
fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
}

impl Product {
    /// Parse from a single product node — a `data.search.products[]` element
    /// or the `data.product` object from a PDP response. RedSky's schema drifts,
    /// so every field beyond `tcin` is best-effort.
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_node_and_unescapes_title() {
        let node = json!({
            "tcin": "80179068",
            "item": {
                "product_description": {"title": "Graco TriRide 3-in-1 &amp; More"},
                "primary_brand": {"name": "Graco"},
                "enrichment": {"buy_url": "https://www.target.com/p/x/-/A-80179068"}
            },
            "price": {"formatted_current_price": "$199.99"}
        });
        let p = Product::from_node(&node).expect("should parse");
        assert_eq!(p.tcin, "80179068");
        assert_eq!(p.title, "Graco TriRide 3-in-1 & More");
        assert_eq!(p.brand.as_deref(), Some("Graco"));
        assert_eq!(p.price.as_deref(), Some("$199.99"));
        assert!(p.url.unwrap().contains("A-80179068"));
    }

    #[test]
    fn missing_tcin_yields_none() {
        assert!(Product::from_node(&json!({"item": {}})).is_none());
    }

    #[test]
    fn tolerates_missing_optional_fields() {
        let p = Product::from_node(&json!({"tcin": "123"})).expect("tcin only is enough");
        assert_eq!(p.tcin, "123");
        assert_eq!(p.title, "");
        assert!(p.brand.is_none());
        assert!(p.url.is_none());
    }
}
