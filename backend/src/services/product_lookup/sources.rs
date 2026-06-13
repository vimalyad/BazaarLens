use serde_json::Value;

use super::{LookupFuture, ProductData, ProductSource};

// ─── Open Food Facts ──────────────────────────────────────────────────────────

pub struct OpenFoodFacts {
    client: reqwest::Client,
}

impl OpenFoodFacts {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl ProductSource for OpenFoodFacts {
    fn lookup<'a>(&'a self, barcode: &'a str) -> LookupFuture<'a> {
        Box::pin(async move {
            let url = format!("https://world.openfoodfacts.org/api/v2/product/{barcode}.json");
            let body: Value = self.client.get(&url).send().await?.json().await?;

            let product = match body.get("product") {
                Some(p) if p.is_object() => p,
                _ => return Ok(None),
            };
            let name = match first_non_empty(&[
                product.get("product_name"),
                product.get("generic_name"),
            ]) {
                Some(n) => n,
                None => return Ok(None),
            };

            Ok(Some(ProductData {
                barcode: Some(barcode.to_string()),
                name,
                brand: first_csv_value(product.get("brands")),
                category: first_csv_value(product.get("categories")),
                image_url: first_non_empty(&[
                    product.get("image_url"),
                    product.get("image_front_url"),
                ]),
                source: "open_food_facts",
                raw: product.clone(),
            }))
        })
    }

    fn source_name(&self) -> &'static str {
        "open_food_facts"
    }
}

// ─── UPC Item DB ──────────────────────────────────────────────────────────────

pub struct UpcItemDb {
    client: reqwest::Client,
}

impl UpcItemDb {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl ProductSource for UpcItemDb {
    fn lookup<'a>(&'a self, barcode: &'a str) -> LookupFuture<'a> {
        Box::pin(async move {
            let url = format!("https://api.upcitemdb.com/prod/trial/lookup?upc={barcode}");
            let body: Value = self.client.get(&url).send().await?.json().await?;

            let item = match body
                .get("items")
                .and_then(|i| i.as_array())
                .and_then(|a| a.first())
            {
                Some(item) => item,
                None => return Ok(None),
            };
            let name = match first_non_empty(&[item.get("title")]) {
                Some(n) => n,
                None => return Ok(None),
            };
            let image_url = item
                .get("images")
                .and_then(|i| i.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(str::to_string);

            Ok(Some(ProductData {
                barcode: Some(barcode.to_string()),
                name,
                brand: first_non_empty(&[item.get("brand")]),
                category: first_csv_value(item.get("category")),
                image_url,
                source: "upc_db",
                raw: item.clone(),
            }))
        })
    }

    fn source_name(&self) -> &'static str {
        "upc_db"
    }
}

// ─── Shared parsing helpers ───────────────────────────────────────────────────

/// First JSON string value that is present and non-empty, trimmed.
fn first_non_empty(values: &[Option<&Value>]) -> Option<String> {
    values
        .iter()
        .filter_map(|v| v.and_then(|v| v.as_str()))
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map(str::to_string)
}

/// First entry of a comma-separated string field (e.g. "Snacks, Chips" → "Snacks").
fn first_csv_value(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| s.split(',').map(str::trim).find(|p| !p.is_empty()))
        .map(str::to_string)
}
