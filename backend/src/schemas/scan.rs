use serde::{Deserialize, Serialize};

/// `POST /api/scan/barcode` request body.
#[derive(Debug, Deserialize)]
pub struct BarcodeScanRequest {
    pub barcode: String,
}

/// `POST /api/scan/image` request body. `hint` optionally constrains the vision model.
#[derive(Debug, Deserialize)]
pub struct ImageScanRequest {
    pub image_base64: String,
    #[serde(default)]
    pub hint: Option<String>,
}

/// Identified product returned to the client. Serialized directly as the `data`
/// field of the API envelope (see `docs/api.md`). `confidence` is only present for
/// vision-identified products; `cached` reports whether this came from the local DB.
#[derive(Debug, Clone, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub barcode: Option<String>,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub source: String,
    pub cached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    pub created_at: String,
}
