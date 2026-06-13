use serde::{Deserialize, Serialize};

use super::scan::ProductResponse;

/// `POST /api/watchlist` request body.
#[derive(Debug, Deserialize)]
pub struct WatchlistAddRequest {
    pub product_id: String,
}

/// `POST /api/watchlist` response — `already_watching` lets the client know whether
/// this was a new entry or a no-op (the DB uses INSERT OR IGNORE for idempotency).
#[derive(Debug, Serialize)]
pub struct WatchlistAddResponse {
    pub already_watching: bool,
}

/// One item in the watchlist list, enriched with product details and the latest
/// recommendation level from the intelligence card (if one has been generated).
#[derive(Debug, Serialize)]
pub struct WatchlistItemResponse {
    pub id: String,
    pub product_id: String,
    pub added_at: String,
    pub product: ProductResponse,
    pub recommendation_level: Option<String>,
}

/// `GET /api/watchlist` response.
#[derive(Debug, Serialize)]
pub struct WatchlistListResponse {
    pub items: Vec<WatchlistItemResponse>,
}
