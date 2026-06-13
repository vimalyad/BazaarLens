mod sources;

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use serde_json::Value;
use sqlx::SqlitePool;
use tracing::warn;
use uuid::Uuid;

use crate::error::AppError;
use crate::schemas::scan::ProductResponse;
use sources::{OpenFoodFacts, UpcItemDb};

/// Normalized product data returned by any lookup source before it is persisted.
/// `barcode` is `None` for vision-identified products.
#[derive(Debug, Clone)]
pub struct ProductData {
    pub barcode: Option<String>,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub source: &'static str,
    pub raw: Value,
}

pub type LookupFuture<'a> =
    Pin<Box<dyn Future<Output = anyhow::Result<Option<ProductData>>> + Send + 'a>>;

/// SOLID — Open/Closed: a new barcode source is a new struct implementing this
/// trait. The orchestrator iterates `Box<dyn ProductSource>` without changing.
/// Uses a boxed future (rather than `async fn` in trait) to stay object-safe
/// without pulling in `async-trait`.
pub trait ProductSource: Send + Sync {
    fn lookup<'a>(&'a self, barcode: &'a str) -> LookupFuture<'a>;
    fn source_name(&self) -> &'static str;
}

/// Tries the local cache first, then each source in registration order.
pub struct ProductLookupService {
    sources: Vec<Box<dyn ProductSource>>,
}

impl Default for ProductLookupService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductLookupService {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client builds with valid defaults");

        Self {
            sources: vec![
                Box::new(OpenFoodFacts::new(client.clone())),
                Box::new(UpcItemDb::new(client)),
            ],
        }
    }

    /// Returns the product (cached or freshly looked up) or `None` if no source
    /// recognises the barcode. Source-level errors are logged and skipped so one
    /// flaky upstream does not fail the whole lookup.
    pub async fn lookup(
        &self,
        pool: &SqlitePool,
        barcode: &str,
    ) -> Result<Option<ProductResponse>, AppError> {
        if let Some(cached) = self.cache_get(pool, barcode).await? {
            return Ok(Some(cached));
        }

        for source in &self.sources {
            match source.lookup(barcode).await {
                Ok(Some(data)) => return Ok(Some(persist(pool, data).await?)),
                Ok(None) => continue,
                Err(e) => {
                    warn!(source = source.source_name(), error = %e, "product source failed");
                    continue;
                }
            }
        }
        Ok(None)
    }

    async fn cache_get(
        &self,
        pool: &SqlitePool,
        barcode: &str,
    ) -> Result<Option<ProductResponse>, AppError> {
        let row = sqlx::query_as::<_, ProductRow>(
            "SELECT id, barcode, name, brand, category, image_url, source, created_at \
             FROM products WHERE barcode = ?",
        )
        .bind(barcode)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|r| r.into_response(true)))
    }
}

/// Inserts a product into the cache and returns its response. Public so the scan
/// route can persist vision-identified products through the same code path.
pub async fn persist(pool: &SqlitePool, data: ProductData) -> Result<ProductResponse, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let raw = data.raw.to_string();

    sqlx::query(
        "INSERT INTO products (id, barcode, name, brand, category, image_url, source, raw_data, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&data.barcode)
    .bind(&data.name)
    .bind(&data.brand)
    .bind(&data.category)
    .bind(&data.image_url)
    .bind(data.source)
    .bind(&raw)
    .bind(&created_at)
    .bind(&created_at)
    .execute(pool)
    .await?;

    Ok(ProductResponse {
        id,
        barcode: data.barcode,
        name: data.name,
        brand: data.brand,
        category: data.category,
        image_url: data.image_url,
        source: data.source.to_string(),
        cached: false,
        confidence: None,
        created_at,
    })
}

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: String,
    barcode: Option<String>,
    name: String,
    brand: Option<String>,
    category: Option<String>,
    image_url: Option<String>,
    source: String,
    created_at: String,
}

impl ProductRow {
    fn into_response(self, cached: bool) -> ProductResponse {
        ProductResponse {
            id: self.id,
            barcode: self.barcode,
            name: self.name,
            brand: self.brand,
            category: self.category,
            image_url: self.image_url,
            source: self.source,
            cached,
            confidence: None,
            created_at: self.created_at,
        }
    }
}
