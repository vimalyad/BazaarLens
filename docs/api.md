# BazaarLens — API Contract Specification

Backend base URL: `https://bazaarlens-api.railway.app`
All responses follow the envelope: `{ "success": bool, "data": any | null, "error": string | null }`

---

## Health

### GET /health
Deployment health check. Called by Railway/Render to verify the container is alive.

**Request:** none

**Response 200:**
```json
{ "status": "ok", "version": "1.0.0" }
```

---

## Scan

### POST /api/scan/barcode
Look up a product by barcode. Checks Supabase cache first, then Open Food Facts, then UPC Item DB.
Does NOT call vision LLM — barcode only.

**Request:**
```json
{ "barcode": "8901030677205" }
```

**Response 200 — cache hit or successful lookup:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "barcode": "8901030677205",
    "name": "Maggi 2-Minute Noodles Masala",
    "brand": "Maggi",
    "category": "Instant Noodles",
    "image_url": "https://images.openfoodfacts.org/...",
    "source": "open_food_facts",
    "cached": false
  },
  "error": null
}
```

**Response 404 — product not found, image needed:**
```json
{
  "success": false,
  "data": null,
  "error": "PRODUCT_NOT_FOUND"
}
```

**Response 422 — invalid barcode format:**
```json
{
  "success": false,
  "data": null,
  "error": "INVALID_BARCODE"
}
```

---

### POST /api/scan/image
Identify a product from an image using Minimax M3 vision. Used when barcode scan fails or user uploads a photo.

**Request:**
```json
{
  "image_base64": "data:image/jpeg;base64,/9j/4AAQSkZJRgAB...",
  "hint": "snack food"
}
```
`hint` is optional — helps constrain the model.

**Response 200:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "barcode": null,
    "name": "Lay's Magic Masala Chips",
    "brand": "Lay's",
    "category": "Snacks",
    "image_url": null,
    "source": "vision_llm",
    "cached": false,
    "confidence": 0.87
  },
  "error": null
}
```

**Response 422 — image too large or unreadable:**
```json
{
  "success": false,
  "data": null,
  "error": "IMAGE_UNPROCESSABLE"
}
```

**Response 503 — vision LLM unavailable:**
```json
{
  "success": false,
  "data": null,
  "error": "VISION_UNAVAILABLE"
}
```

---

## Intelligence

### POST /api/intelligence
Generate an intelligence card for a product. Checks Supabase cache first (by product_id). If cached, returns immediately. If not, calls DeepSeek V4 Pro and writes result to DB.

**Request:**
```json
{ "product_id": "550e8400-e29b-41d4-a716-446655440000" }
```

**Response 200:**
```json
{
  "success": true,
  "data": {
    "id": "660e8400-e29b-41d4-a716-446655440000",
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "pricing_insight": "Competitors on Amazon selling at ₹28–₹32 per pack; this product is currently priced at ₹30, sitting in the mid-range. There is room for a 10% premium if brand recall is leveraged.",
    "review_insight": "Customers consistently rate this product 4.1+ for taste but flag the smaller pack size as a concern. Opportunity to differentiate on value-per-gram messaging.",
    "market_insight": "Instant noodle category saw 23% demand growth post-2025 across Tier-2 cities. This SKU has strong recall in North and Central India.",
    "recommendation": "List at ₹29 with a combo bundle (2+1) to increase cart value and counter private-label competitors gaining ground on Meesho.",
    "recommendation_level": "buy",
    "confidence": 0.84,
    "cached": false,
    "generated_at": "2026-06-13T10:30:00Z"
  },
  "error": null
}
```

**Response 404 — product_id not found:**
```json
{
  "success": false,
  "data": null,
  "error": "PRODUCT_NOT_FOUND"
}
```

**Response 503 — LLM call failed after retries:**
```json
{
  "success": false,
  "data": null,
  "error": "INTELLIGENCE_GENERATION_FAILED"
}
```

---

## Watchlist

### POST /api/watchlist
Add a product to device watchlist. Idempotent — duplicate adds return the existing row.

**Headers:** `X-Device-Id: <uuid>`

**Request:**
```json
{ "product_id": "550e8400-e29b-41d4-a716-446655440000" }
```

**Response 200:**
```json
{
  "success": true,
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440000",
    "device_id": "abc123def456",
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "product_name": "Maggi 2-Minute Noodles Masala",
    "product_image_url": "https://...",
    "product_category": "Instant Noodles",
    "added_at": "2026-06-13T10:31:00Z",
    "already_watching": false
  },
  "error": null
}
```

**Response 422 — missing device_id header:**
```json
{
  "success": false,
  "data": null,
  "error": "DEVICE_ID_REQUIRED"
}
```

---

### GET /api/watchlist
Get all products on device watchlist. Returns enriched rows including product details.

**Headers:** `X-Device-Id: <uuid>`

**Response 200:**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "770e8400-e29b-41d4-a716-446655440000",
        "product_id": "550e8400-e29b-41d4-a716-446655440000",
        "product_name": "Maggi 2-Minute Noodles Masala",
        "product_image_url": "https://...",
        "product_category": "Instant Noodles",
        "product_brand": "Maggi",
        "recommendation_level": "buy",
        "added_at": "2026-06-13T10:31:00Z"
      }
    ],
    "count": 1
  },
  "error": null
}
```

**Response 200 — empty watchlist:**
```json
{
  "success": true,
  "data": { "items": [], "count": 0 },
  "error": null
}
```

---

### DELETE /api/watchlist/{product_id}
Remove a product from watchlist.

**Headers:** `X-Device-Id: <uuid>`

**Response 200:**
```json
{
  "success": true,
  "data": { "removed": true },
  "error": null
}
```

**Response 404 — not on watchlist:**
```json
{
  "success": false,
  "data": null,
  "error": "NOT_ON_WATCHLIST"
}
```

---

## Market Events

### POST /api/events/simulate
Trigger a simulated market event for a product. Admin-only in spirit, no auth in MVP.
This endpoint: (1) writes the event to DB, (2) generates a strategy, (3) sends push notifications to all watchers.

**Request:**
```json
{
  "product_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "price_drop",
  "metadata": {
    "competitor_name": "Sunfeast YiPPee!",
    "old_price": 35,
    "new_price": 28,
    "drop_percentage": 20
  }
}
```

`event_type` values: `price_drop` | `demand_spike` | `sentiment_crash` | `stock_outage`

**Response 200:**
```json
{
  "success": true,
  "data": {
    "event_id": "880e8400-e29b-41d4-a716-446655440000",
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "price_drop",
    "notifications_sent": 2,
    "strategy_id": "990e8400-e29b-41d4-a716-446655440000",
    "triggered_at": "2026-06-13T10:35:00Z"
  },
  "error": null
}
```

**Response 404 — product not found:**
```json
{
  "success": false,
  "data": null,
  "error": "PRODUCT_NOT_FOUND"
}
```

---

## Strategy

### POST /api/strategy
Generate or retrieve strategy for a market event. Checks cache (strategies table) first.

**Request:**
```json
{ "event_id": "880e8400-e29b-41d4-a716-446655440000" }
```

**Response 200:**
```json
{
  "success": true,
  "data": {
    "id": "990e8400-e29b-41d4-a716-446655440000",
    "event_id": "880e8400-e29b-41d4-a716-446655440000",
    "product_name": "Maggi 2-Minute Noodles Masala",
    "event_type": "price_drop",
    "event_summary": "Sunfeast YiPPee! dropped price by 20% (₹35 → ₹28)",
    "marketing_ai": {
      "opinion": "This is an opportunity, not a threat. YiPPee's aggressive pricing signals desperation for market share.",
      "action": "Launch a 'Trusted Since 1983' brand story campaign on Instagram Reels targeting 25-40 age group.",
      "urgency": "medium"
    },
    "product_ai": {
      "opinion": "Price alone cannot unseat a category leader. Focus on the emotional association Maggi has built.",
      "action": "Introduce a limited edition 'Nostalgia Pack' with retro packaging to reinforce brand equity.",
      "urgency": "low"
    },
    "sales_ai": {
      "opinion": "A 20% price drop by a competitor will move price-sensitive buyers. Act within 48 hours.",
      "action": "Offer a 3-pack combo at ₹80 (effective ₹26.6/pack) on Meesho for 7 days only.",
      "urgency": "high"
    },
    "final_decision": {
      "recommended_action": "Launch 3-pack combo at ₹80 on Meesho immediately, combined with a 48-hour flash sale banner.",
      "reasoning": "Price-sensitive Meesho buyers will respond to the combo value. This defends volume without a permanent price cut that damages brand perception.",
      "risk_assessment": "If not acted within 48 hours, Meesho algorithm will boost YiPPee listings due to conversion rate improvement.",
      "time_sensitivity": "Act within 24 hours"
    },
    "cached": false,
    "generated_at": "2026-06-13T10:35:05Z"
  },
  "error": null
}
```

**Response 404 — event not found:**
```json
{
  "success": false,
  "data": null,
  "error": "EVENT_NOT_FOUND"
}
```

---

## Push Notifications

### POST /api/push/subscribe
Save a push subscription for a device. Upserts on device_id — handles subscription refresh.

**Headers:** `X-Device-Id: <uuid>`

**Request:**
```json
{
  "subscription": {
    "endpoint": "https://fcm.googleapis.com/fcm/send/...",
    "keys": {
      "p256dh": "BNcRdreALRFXTkOOUHK...",
      "auth": "tBHItJI5svbpez7KI4..."
    }
  }
}
```

**Response 200:**
```json
{
  "success": true,
  "data": { "subscribed": true },
  "error": null
}
```

**Response 422 — invalid subscription object:**
```json
{
  "success": false,
  "data": null,
  "error": "INVALID_SUBSCRIPTION"
}
```

---

### DELETE /api/push/subscribe
Unsubscribe a device from push notifications.

**Headers:** `X-Device-Id: <uuid>`

**Response 200:**
```json
{
  "success": true,
  "data": { "unsubscribed": true },
  "error": null
}
```

---

## Error Code Reference

| Code | Meaning |
|---|---|
| `PRODUCT_NOT_FOUND` | Barcode/ID not found in any source |
| `INVALID_BARCODE` | Barcode format validation failed |
| `IMAGE_UNPROCESSABLE` | Image too large, corrupt, or unreadable |
| `VISION_UNAVAILABLE` | Minimax M3 API unavailable after retries |
| `INTELLIGENCE_GENERATION_FAILED` | DeepSeek API failed after retries |
| `DEVICE_ID_REQUIRED` | X-Device-Id header missing |
| `NOT_ON_WATCHLIST` | DELETE attempted on unwatched product |
| `EVENT_NOT_FOUND` | event_id not found in market_events |
| `INVALID_SUBSCRIPTION` | Malformed push subscription object |
| `STRATEGY_GENERATION_FAILED` | DeepSeek API failed during strategy gen |
