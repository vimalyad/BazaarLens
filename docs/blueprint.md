# BazaarLens — Engineering Blueprint

Complete implementation plan. No application code. Guide for the full 30-hour build.

---

# 1. Architecture Validation

## 1.1 Frontend Architecture

**Stack:** Next.js 14 (App Router) + TypeScript + TailwindCSS + shadcn/ui + PWA

### What is Good
- App Router is the correct choice for Next.js 14 — enables React Server Components for fast initial paint on intelligence cards
- ZXing replaces BarcodeDetector API correctly — works in all browsers including Safari
- SWR for data fetching is lean and correct for this use case
- shadcn/ui gives polished components without design debt

### What Could Fail

**Problem 1: next-pwa v5 is incompatible with App Router**
next-pwa v5.x was built for the Pages Router. Using it with App Router causes service worker registration failures and build errors.
Fix: Use `@ducanh2912/next-pwa` (actively maintained App Router fork) or `serwist` (Workbox successor).

**Problem 2: ZXing bundle size**
`@zxing/browser` adds ~200KB to the bundle. If loaded eagerly, it slows the initial paint on slow mobile connections.
Fix: Dynamically import `BarcodeScanner.tsx` using `next/dynamic` with `ssr: false`. Only loads when the scan page is active.

**Problem 3: Camera permission UX on iOS**
If a user denies camera permission, there is no browser API to re-prompt. The app silently fails.
Fix: Before initializing ZXing, call `navigator.mediaDevices.getUserMedia()` with a try/catch. If it fails, immediately show the ImageUpload fallback with a "Camera blocked — use photo instead" message.

**Problem 4: iOS standalone mode detection must happen early**
Push notification subscription must be blocked on non-standalone iOS. If not guarded, `pushManager.subscribe()` fails silently.
Fix: In `usePushNotifications.ts`, detect `(display-mode: standalone)` media query before any push call. If not standalone on iOS, show the "Add to Home Screen" guide modal.

**Problem 5: SWR key management**
If `device_id` is generated client-side (localStorage), SWR keys containing device_id will be undefined on SSR, causing hydration mismatches.
Fix: All watchlist SWR fetches must be wrapped in a `useEffect` or use SWR's `null` key pattern (skip fetch until device_id is available): `useSWR(deviceId ? '/api/watchlist' : null, ...)`.

### Adjustments Before Implementation
1. Replace `next-pwa` with `@ducanh2912/next-pwa` in package.json from day one
2. All scanner-related imports must use `next/dynamic`
3. Add `isStandalone()` utility to `lib/utils.ts`
4. SWR keys must handle undefined device_id via null key pattern

---

## 1.2 Backend Architecture

**Stack:** FastAPI + httpx (async) + pywebpush + supabase-py

### What is Good
- FastAPI is async-native — handles concurrent LLM calls and DB operations without blocking
- httpx AsyncClient for all external calls is correct
- Single-service architecture avoids microservice overhead — right for 30 hours

### What Could Fail

**Problem 1: pywebpush is synchronous**
`pywebpush.webpush()` is a blocking synchronous function. Calling it directly in an `async def` route handler blocks the FastAPI event loop, degrading performance under load.
Fix: Wrap every pywebpush call in `asyncio.to_thread(webpush, ...)` to run it in a thread pool without blocking.

**Problem 2: supabase-py async compatibility**
The `supabase-py` client uses `httpx` internally but the Python client's methods are synchronous by default in v2. Calling them in async routes blocks the event loop.
Fix: Use `supabase-py` v2's async client (`AsyncClient`) which uses `httpx.AsyncClient` internally. Instantiate as `acreate_client()`.

**Problem 3: Cold start on Railway/Render**
Railway and Render free/hobby tiers spin containers down after inactivity. First request after a cold start can take 5–10 seconds.
Fix: Add a `/health` endpoint. Configure Railway's health check to ping it every 5 minutes. For the demo, hit the backend API once before presenting to warm the container.

**Problem 4: CORS pre-flight**
Mobile browsers often trigger CORS pre-flight (OPTIONS) before the real request. Without proper CORS handling, all POST requests fail.
Fix: FastAPI `CORSMiddleware` must include `allow_methods=["*"]` and `allow_headers=["*"]` including `X-Device-Id`.

**Problem 5: Image payload size**
Base64-encoded images for vision scan can be 2–5MB per request. Without a size limit, a bad image upload can hang the backend.
Fix: Add FastAPI body size limit of 10MB. Resize images on the frontend before encoding: max 800px width, JPEG quality 80%.

**Problem 6: Event simulator is synchronous in user path**
`POST /api/events/simulate` does 4 things: write event, generate strategy, query watchers, send push to each watcher. If a watcher has an expired push subscription, pywebpush throws and the whole request fails.
Fix: Separate strategy generation and push sending into a background task using FastAPI's `BackgroundTasks`. Return the event_id immediately (within 200ms), then do generation + notification async.

### Adjustments Before Implementation
1. Use `asyncio.to_thread()` for all pywebpush calls
2. Use `supabase-py` v2 AsyncClient
3. Add `/health` endpoint as first route created
4. Add CORSMiddleware immediately in `main.py` setup
5. Add image resize step in frontend before base64 encoding
6. Use `BackgroundTasks` for the simulate endpoint

---

## 1.3 Database Schema

### What is Good
- UUID primary keys prevent enumeration attacks
- JSONB for `raw_data` in products gives flexibility
- Unique constraints on watchlist and push_subscriptions are correct

### What Could Fail

**Problem 1: No index on `device_id` columns**
`GET /api/watchlist` will do a full table scan on the watchlist table since device_id is a plain TEXT column with no index. At demo scale this is fine, but it still adds 50–100ms unnecessarily.
Fix: Add `CREATE INDEX idx_watchlist_device_id ON watchlist(device_id)` and the same for push_subscriptions.

**Problem 2: `intelligence_cards` has no unique constraint on `product_id`**
Running intelligence generation twice for the same product creates duplicate rows. The query logic must be careful, but this is fragile.
Fix: Add `UNIQUE(product_id)` on intelligence_cards. Use `INSERT ... ON CONFLICT (product_id) DO NOTHING` or `DO UPDATE` when caching.

**Problem 3: `strategies` has no unique constraint on `event_id`**
Same issue — duplicate strategy generation can create multiple rows per event.
Fix: Add `UNIQUE(event_id)` on strategies.

**Problem 4: `market_events` has no index on `product_id`**
When simulating an event, we query watchers by product_id → watchlist → device_ids → push_subscriptions. The market_events.product_id lookup needs an index.
Fix: Add `CREATE INDEX idx_market_events_product_id ON market_events(product_id)`.

**Problem 5: No Supabase Row Level Security (RLS) policy**
Without RLS, any client with the anon key can read all data from all devices. For a hackathon demo, this is an edge case, but judges might probe the API directly.
Fix: Enable RLS on all tables with a simple policy: service role (backend) has full access, anon role (frontend) has no direct access. All frontend access goes through the FastAPI backend.

**Problem 6: Push subscription refresh**
When a push subscription expires (common on iOS after 1–2 weeks), the endpoint changes. The Supabase row must be updated.
Fix: The subscribe endpoint uses UPSERT (`ON CONFLICT (device_id) DO UPDATE`) so refreshed subscriptions overwrite stale ones.

**Problem 7: `market_events` needs a `notification_sent` flag**
Without this, re-triggering the event simulator would re-send duplicate push notifications.
Fix: Add `notification_sent BOOLEAN DEFAULT FALSE` to market_events and `notifications_count INT DEFAULT 0`.

---

## 1.4 AI Integration Flow

### What is Good
- Single LLM call per action prevents latency compounding
- `response_format: { type: "json_object" }` is correct
- Caching intelligence and strategy in Supabase prevents re-burning credits

### What Could Fail

**Problem 1: OpenRouter model IDs are not verified**
CLAUDE.md lists `deepseek/deepseek-chat-v4` and `minimax/minimax-m3` as model IDs. These may not match actual OpenRouter slugs.
Fix: Before writing any LLM code, call `GET https://openrouter.ai/api/v1/models` with the API key to list all available models. Map the correct slugs. Do this in the first 30 minutes.

**Problem 2: No JSON schema validation**
If the LLM returns `{}` or a partial JSON that Pydantic can't parse, the endpoint crashes with a 500.
Fix: Every LLM response goes through Pydantic validation. If validation fails, trigger one automatic retry with a stricter prompt. If retry fails, return the `*_GENERATION_FAILED` error code.

**Problem 3: `response_format: json_object` requires at least one `json` mention in the prompt**
OpenAI-compatible APIs (including OpenRouter) require the word "json" or "JSON" to appear in the prompt when `response_format: json_object` is set. Otherwise, some models return an error.
Fix: All prompts must end with `Return ONLY valid JSON:` or similar.

**Problem 4: Temperature 0.3 might be too high for strict JSON**
Some models at temperature > 0 still occasionally add markdown code fences (```json ... ```) around the response, breaking JSON parsing.
Fix: Strip markdown fences before `json.loads()`. Add a `clean_json_response(text: str) -> str` utility that strips leading/trailing ``` and whitespace.

**Problem 5: Timeout**
OpenRouter can take 15–45 seconds for complex prompts on free-tier models. FastAPI's default timeout is unlimited, but Vercel's edge has a 30s limit, and Railway has no limit.
Fix: Set `httpx.AsyncClient(timeout=30.0)` on all LLM calls. If timeout hits, return a graceful error rather than crashing.

### Adjustments Before Implementation
1. Verify all model IDs against OpenRouter catalog BEFORE writing LLM code
2. Add `clean_json_response()` utility
3. Add Pydantic models for every LLM response shape
4. Set 30s timeout on the httpx LLM client
5. Add retry logic: 1 automatic retry on validation failure, not on timeout

---

## 1.5 Push Notification Flow

### What is Good
- VAPID-based Web Push is the correct standard
- pywebpush is the right Python library

### What Could Fail

**Problem 1: VAPID key pair generation**
VAPID keys must be generated once and stored as environment variables. If generated at runtime, the keys change on every restart and all existing subscriptions become invalid.
Fix: Generate keys ONCE during project setup using `py-vapid` CLI:
```
vapid --gen --applicationServerKey
```
Store `VAPID_PRIVATE_KEY` and `VAPID_PUBLIC_KEY` in Railway env vars. Never regenerate.

**Problem 2: iOS push notifications expire**
iOS push subscriptions can silently expire after 1–2 weeks. `pywebpush` returns a 410 Gone HTTP status when this happens.
Fix: When pywebpush returns 410, delete the push_subscription row from Supabase and log it. Do not crash.

**Problem 3: VAPID public key format mismatch**
The frontend needs the VAPID public key in URL-safe base64 format (applicationServerKey). The key generated by py-vapid may be in a different format.
Fix: Use `urlsafe_b64encode(vapid_private_key.get_public_key())` to produce the correct format for the frontend's `pushManager.subscribe()` call.

**Problem 4: Permission prompt not shown if called outside user gesture**
iOS Safari strictly silences `Notification.requestPermission()` if not called from a click handler.
Fix: The "Enable Alerts" button in `usePushNotifications.ts` must call `requestPermission()` directly in the `onClick` handler, not in a setTimeout, not in a useEffect.

---

## 1.6 Event Simulator Flow

### What is Good
- Hidden `/admin` page is simple and demo-reliable
- Event types cover the most compelling demo scenarios

### What Could Fail

**Problem 1: Simulate endpoint does too much synchronously**
Current design: write event → generate strategy → find watchers → send pushes → return response. All synchronous. If strategy generation takes 20s and there are 3 watchers with slow push endpoints, the HTTP request times out.
Fix: Return the event_id immediately. Use FastAPI `BackgroundTasks` for strategy generation and push delivery.

**Problem 2: Admin page is accessible to anyone**
During the demo, if someone else in the room opens the admin page, they can trigger events on your demo device.
Fix: Add a query-param secret: `/admin?key=demo2026`. Check this in the component — if key doesn't match, show 404. Not security, just collision prevention.

---

# 2. End-to-End User Flow

## Flow A: Scan → Intelligence → Watchlist

```
USER ACTION                     FRONTEND                    BACKEND                         DATABASE
─────────────────────────────────────────────────────────────────────────────────────────────────────
Opens /scan                     Renders ScanPage            -                               -
                                Requests camera permission  -                               -
                                ZXing starts scanning       -                               -
                                
[Barcode detected]              barcode: "8901030677205"    -                               -
                                Shows scanning flash        -                               -
                                → POST /api/scan/barcode    Receives {barcode}              -
                                                            Check products WHERE barcode=?  products (READ)
                                                            [MISS] Call Open Food Facts     External API
                                                            [HIT] Return cached product     -
                                                            [MISS] Call UPC Item DB         External API
                                                            Write to products table         products (WRITE)
                                                            Return Product object           -
                                Receives product            -                               -
                                Shows ScanResult bottom     -                               -
                                sheet (name, brand, image)  -                               -
                                
User taps "See Intelligence"    Navigate to /product/[id]   -                               -
                                Shows skeleton card         -                               -
                                → POST /api/intelligence    Receives {product_id}           -
                                                            Check intelligence_cards        intelligence_cards (READ)
                                                            [MISS] Build prompt             -
                                                            Call DeepSeek V4 Pro            OpenRouter API
                                                            Parse + validate JSON           -
                                                            Write to intelligence_cards     intelligence_cards (WRITE)
                                                            Return IntelligenceCard         -
                                Animate card reveal         -                               -
                                Show 4 insight panels       -                               -
                                Show recommendation badge   -                               -
                                
User taps "Add to Watchlist"    → POST /api/watchlist       Receives {product_id}           -
  button                          + X-Device-Id header      Check for duplicate             watchlist (READ)
                                                            INSERT into watchlist           watchlist (WRITE)
                                                            Return success                  -
                                Shows success toast         -                               -
                                Button changes to "Watching"-                               -
```

## Flow B: Image Upload Fallback (no barcode)

```
USER ACTION                     FRONTEND                    BACKEND                         EXTERNAL
─────────────────────────────────────────────────────────────────────────────────────────────────────
Taps "Upload Photo" on /scan    Shows ImageUpload picker    -                               -
Selects image                   Resize to max 800px         -                               -
                                Convert to base64           -                               -
                                → POST /api/scan/image      Receives {image_base64}         -
                                                            Call Minimax M3 vision          OpenRouter API
                                                            Parse product details from resp -
                                                            Write to products table         products (WRITE)
                                                            Return Product + confidence     -
                                Shows product with          -                               -
                                "AI identified" badge       -                               -
                                (if confidence < 0.7:       -                               -
                                 show "Low confidence"      -                               -
                                 warning)                   -                               -
```

## Flow C: Market Event → Push → Strategy

```
ADMIN ACTION                    ADMIN PAGE                  BACKEND                         DATABASE / PUSH
─────────────────────────────────────────────────────────────────────────────────────────────────────
Opens /admin?key=demo2026       Loads watchlist products    → GET /api/watchlist            watchlist (READ)
                                Shows product dropdown      -                               -
Selects product + event type    -                           -                               -
Taps "Trigger Event"            → POST /api/events/simulate Receives {product_id,           -
                                                             event_type, metadata}
                                                            Write to market_events          market_events (WRITE)
                                                            Return event_id IMMEDIATELY     -
                                Shows "Event triggered!"    [BACKGROUND TASK STARTS]        -
                                                            Build strategy prompt           -
                                                            Call DeepSeek V4 Pro            OpenRouter API
                                                            Validate + write strategy       strategies (WRITE)
                                                            Query watchlist for watchers    watchlist (READ)
                                                            Query push subscriptions        push_subscriptions (READ)
                                                            For each subscription:          -
                                                              asyncio.to_thread(webpush)    Web Push Protocol
                                                              If 410: delete subscription   push_subscriptions (DELETE)
                                
[USER'S PHONE]
Receives push notification      Service worker fires        -                               -
"push" event                    showNotification() called   -                               -
Notification displayed          Title: "Market Alert!"      -                               -
                                Body: "Competitor dropped   -                               -
                                price 20% — see strategy"   -                               -
                                
User taps notification          Service worker handles      -                               -
                                "notificationclick"         -                               -
                                Opens /strategy/[eventId]   -                               -
                                Shows AI thinking anim      -                               -
                                → POST /api/strategy        Receives {event_id}             -
                                                            Check strategies table          strategies (READ)
                                                            [HIT] Return cached strategy    -
                                                            (strategy was generated in      -
                                                            background, so it's ready)      -
                                Renders 3 AI opinion cards  -                               -
                                Renders final decision card -                               -
                                Shows "Share Report" button -                               -
                                
User taps "Share Report"        navigator.share() called    -                               -
                                iOS share sheet opens       -                               -
                                [iQOO Office Kit moment]    -                               -
```

---

# 3. API Contract Specification

See `docs/api.md` for the complete production-ready API contract.

Summary of all endpoints:

| Method | Route | Purpose |
|---|---|---|
| GET | /health | Deployment health check |
| POST | /api/scan/barcode | Look up product by barcode |
| POST | /api/scan/image | Identify product via vision LLM |
| POST | /api/intelligence | Generate/retrieve intelligence card |
| POST | /api/watchlist | Add product to watchlist |
| GET | /api/watchlist | Get all watched products |
| DELETE | /api/watchlist/{product_id} | Remove from watchlist |
| POST | /api/events/simulate | Trigger market event (admin) |
| POST | /api/strategy | Generate/retrieve strategy |
| POST | /api/push/subscribe | Save push subscription |
| DELETE | /api/push/subscribe | Remove push subscription |

---

# 4. Database — Final Production Schema

## Final Migration SQL

```sql
-- Enable UUID extension (Supabase has this by default)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================
-- TABLE: products
-- Cache of identified products from Open Food Facts / vision
-- ============================================================
CREATE TABLE products (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  barcode      TEXT UNIQUE,                    -- NULL for vision-identified products
  name         TEXT NOT NULL,
  brand        TEXT,
  category     TEXT,
  image_url    TEXT,
  source       TEXT NOT NULL DEFAULT 'unknown', -- 'open_food_facts' | 'upc_db' | 'vision_llm'
  raw_data     JSONB,                           -- full API response for debugging
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_barcode ON products(barcode) WHERE barcode IS NOT NULL;
CREATE INDEX idx_products_category ON products(category);

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;
CREATE TRIGGER products_updated_at BEFORE UPDATE ON products
  FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- TABLE: intelligence_cards
-- AI-generated intelligence per product (cached)
-- ============================================================
CREATE TABLE intelligence_cards (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id           UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  pricing_insight      TEXT NOT NULL,
  review_insight       TEXT NOT NULL,
  market_insight       TEXT NOT NULL,
  recommendation       TEXT NOT NULL,
  recommendation_level TEXT NOT NULL CHECK (recommendation_level IN ('buy','hold','avoid','watch')),
  confidence           NUMERIC(3,2) CHECK (confidence >= 0 AND confidence <= 1),
  model_used           TEXT,                   -- for debugging, tracks which model generated this
  generated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(product_id)                           -- one card per product, refresh via UPDATE
);

CREATE INDEX idx_intelligence_cards_product_id ON intelligence_cards(product_id);

-- ============================================================
-- TABLE: watchlist
-- Device-based watchlist (no user auth required)
-- ============================================================
CREATE TABLE watchlist (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  device_id    TEXT NOT NULL CHECK (length(device_id) = 36), -- UUID format enforcement
  product_id   UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  added_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(device_id, product_id)
);

CREATE INDEX idx_watchlist_device_id ON watchlist(device_id);
CREATE INDEX idx_watchlist_product_id ON watchlist(product_id);

-- ============================================================
-- TABLE: push_subscriptions
-- Web Push VAPID subscriptions per device
-- ============================================================
CREATE TABLE push_subscriptions (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  device_id    TEXT NOT NULL CHECK (length(device_id) = 36),
  endpoint     TEXT NOT NULL,
  p256dh       TEXT NOT NULL,
  auth_key     TEXT NOT NULL,                  -- renamed from 'auth' to avoid SQL keyword collision
  is_active    BOOLEAN NOT NULL DEFAULT TRUE,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(device_id)                            -- one subscription per device; upserted on refresh
);

CREATE INDEX idx_push_subscriptions_device_id ON push_subscriptions(device_id);
CREATE TRIGGER push_subscriptions_updated_at BEFORE UPDATE ON push_subscriptions
  FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- TABLE: market_events
-- Simulated market events that trigger notifications
-- ============================================================
CREATE TABLE market_events (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id          UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  event_type          TEXT NOT NULL CHECK (event_type IN ('price_drop','demand_spike','sentiment_crash','stock_outage')),
  event_data          JSONB NOT NULL DEFAULT '{}',   -- {competitor_name, old_price, new_price, ...}
  notification_sent   BOOLEAN NOT NULL DEFAULT FALSE,
  notifications_count INT NOT NULL DEFAULT 0,
  triggered_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_market_events_product_id ON market_events(product_id);
CREATE INDEX idx_market_events_triggered_at ON market_events(triggered_at DESC);

-- ============================================================
-- TABLE: strategies
-- AI-generated strategies per market event (cached)
-- ============================================================
CREATE TABLE strategies (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id         UUID NOT NULL REFERENCES market_events(id) ON DELETE CASCADE,
  marketing_ai     JSONB NOT NULL,             -- {opinion, action, urgency}
  product_ai       JSONB NOT NULL,             -- {opinion, action, urgency}
  sales_ai         JSONB NOT NULL,             -- {opinion, action, urgency}
  final_decision   JSONB NOT NULL,             -- {recommended_action, reasoning, risk_assessment, time_sensitivity}
  model_used       TEXT,
  generated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(event_id)                             -- one strategy per event
);

CREATE INDEX idx_strategies_event_id ON strategies(event_id);

-- ============================================================
-- ROW LEVEL SECURITY
-- Frontend anon key has no direct table access.
-- All reads/writes go through FastAPI (service role key).
-- ============================================================
ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE intelligence_cards ENABLE ROW LEVEL SECURITY;
ALTER TABLE watchlist ENABLE ROW LEVEL SECURITY;
ALTER TABLE push_subscriptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE market_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE strategies ENABLE ROW LEVEL SECURITY;

-- Service role (used by backend) has full access — Supabase grants this by default
-- Anon role has no access — this is the default when RLS is enabled with no policies
```

## Table Relationship Summary

```
products (1) ──── (0..1) intelligence_cards
products (1) ──── (0..N) watchlist
products (1) ──── (0..N) market_events
market_events (1) ──── (0..1) strategies
watchlist.device_id ←→ push_subscriptions.device_id  [not a FK, just a matching key]
```

---

# 5. Frontend Screen Breakdown

## /scan

**Purpose:** Primary entry point. Camera scanner with barcode detection.

**Components:**
- `ScanPage` (page.tsx) — orchestrates scanner state machine
- `BarcodeScanner` — ZXing camera feed, overlay, detection
- `ImageUpload` — fallback file picker
- `ScanResult` — bottom sheet showing identified product
- `CameraPermissionDenied` — fallback state when camera is blocked
- `AddToHomeScreenModal` — iOS guide for push setup (triggered once)

**State Machine:**
```
IDLE → SCANNING → DETECTED → IDENTIFYING → IDENTIFIED → REDIRECT
               ↓ (camera denied)
            UPLOAD_FALLBACK → IDENTIFYING → IDENTIFIED → REDIRECT
```

**Data Requirements:** None on load (no server data needed)

**API Calls:**
- `POST /api/scan/barcode` when barcode detected
- `POST /api/scan/image` when image uploaded

**Loading State:** ZXing camera preview is the loading state — shows immediately.

**Error States:**
- Camera permission denied → Show ImageUpload with explanation
- Product not found after barcode lookup → Show "Can't identify this product" with retry + image upload option
- Network error → Toast: "Check your connection"

---

## /product/[id]

**Purpose:** Display AI-generated intelligence card for an identified product.

**Components:**
- `ProductPage` (page.tsx) — fetches + renders
- `IntelligenceCard` — full-screen card container with reveal animation
- `PricingInsight` — insight panel (icon + text)
- `ReviewInsight` — insight panel
- `MarketInsight` — insight panel
- `RecommendationBadge` — colored badge (buy=green, hold=yellow, avoid=red, watch=blue) with pulsing ring animation
- `AddToWatchlistButton` — sticky bottom bar button
- `ShareButton` — Web Share API trigger (for Office Kit scoring)
- `IntelligenceCardSkeleton` — while AI generates

**Data Requirements:**
- `product_id` from URL param
- Product details (name, brand, category, image) — fetched from `/api/scan/barcode` result stored in sessionStorage, or re-fetched
- Intelligence card from API

**API Calls:**
- `POST /api/intelligence { product_id }`
- `POST /api/watchlist { product_id }` on CTA tap

**Loading State:** `IntelligenceCardSkeleton` — animated gray pulses for each insight panel while AI generates (looks like AI "thinking"). Should take 3–8 seconds.

**Error States:**
- Intelligence generation failed → "Our AI is thinking hard. Try again?" with retry button
- Product not found → Redirect to /scan

**Notes:**
- Product data should be passed via query params or sessionStorage from /scan to avoid a second API round-trip
- URL should be shareable: `/product/[id]` should work if opened fresh (fetch product on mount)

---

## /watchlist

**Purpose:** Show all monitored products. Entry point for enabling push notifications.

**Components:**
- `WatchlistPage` (page.tsx)
- `WatchlistCard` — product card with name, image, category, recommendation badge, remove button
- `EnableAlertsCard` — prominent card at top if push not yet enabled (most important CTAs)
- `AddToHomeScreenModal` — iOS instructions modal (if not standalone)
- `EmptyWatchlist` — empty state with CTA to scan

**Data Requirements:**
- `device_id` from localStorage
- Push subscription status from `usePushNotifications`
- Watchlist items from API

**API Calls:**
- `GET /api/watchlist` (with X-Device-Id header)
- `DELETE /api/watchlist/{product_id}` on remove
- `POST /api/push/subscribe` on enable alerts

**Loading State:** Skeleton cards for each item

**Error States:**
- Empty watchlist → Large empty state graphic + "Scan your first product" CTA
- Push denied → Persistent "Notifications blocked — open Settings to re-enable" banner

---

## /strategy/[eventId]

**Purpose:** Show multi-perspective AI strategy triggered by a market event.

**Components:**
- `StrategyPage` (page.tsx)
- `EventBanner` — shows the triggering event (e.g., "Competitor dropped price 20%") prominently at top
- `AIOpinionCard` — card for each AI perspective (marketing, product, sales)
  - Shows opinion, action, urgency badge (high=red, medium=yellow, low=green)
- `FinalDecisionCard` — highlighted card with recommended action, reasoning, risk, time sensitivity
- `StrategyLoadingState` — animated "AI team analyzing..." state with 3 pulsing dots
- `ShareButton` — Web Share API

**Data Requirements:**
- `eventId` from URL param
- Strategy from API (may be cached already from background generation)

**API Calls:**
- `POST /api/strategy { event_id }`

**Loading State:** "Your AI team is analyzing the situation..." with animated consultant icons. Should be brief since strategy is generated in background during notification delivery.

**Error States:**
- Event not found → "This alert has expired" with back to watchlist button
- Strategy generation failed → Retry button

---

## /admin

**Purpose:** Demo trigger for market events. Minimal UI — internal tool.

**Components:**
- `AdminPage` (page.tsx)
- Plain HTML form (no shadcn/ui needed — judges won't see this page)
- Product selector (dropdown from watchlist)
- Event type selector (4 options)
- Metadata fields (pre-filled with sensible defaults)
- "Trigger Event" button
- Result display (event_id + notifications sent count)

**Access Control:** Query param `?key=demo2026`. If missing or wrong, render a blank page.

**API Calls:**
- `GET /api/watchlist` to populate product dropdown
- `POST /api/events/simulate`

**Loading State:** Button shows "Triggering..." spinner

**Error States:** Inline error message if simulate fails

---

# 6. Component Build Order

## Complete Component List

**Layout / Foundation:**
- `app/layout.tsx` — root layout with PWA meta tags
- `components/ui/` — shadcn/ui primitives (Button, Card, Badge, Sheet, Toast, Skeleton, Modal)
- `components/BottomNav.tsx` — bottom navigation bar (Scan, Watchlist tabs)
- `lib/utils.ts` — `cn()`, `isIOS()`, `isStandalone()`, `getDeviceId()`
- `lib/api.ts` — typed fetch wrapper for all backend endpoints
- `lib/supabase.ts` — Supabase client (read-only, frontend uses backend for all writes)
- `types/index.ts` — all shared TypeScript types

**Hooks:**
- `hooks/useDeviceId.ts` — localStorage device_id generation + retrieval
- `hooks/useScanner.ts` — ZXing camera initialization + decoding
- `hooks/usePushNotifications.ts` — subscription flow + iOS standalone guard
- `hooks/useWatchlist.ts` — SWR-based watchlist CRUD

**Scan Components:**
- `components/scan/BarcodeScanner.tsx`
- `components/scan/ImageUpload.tsx`
- `components/scan/ScanResult.tsx`
- `components/scan/CameraPermissionDenied.tsx`

**Intelligence Components:**
- `components/intelligence/IntelligenceCard.tsx`
- `components/intelligence/IntelligenceCardSkeleton.tsx`
- `components/intelligence/PricingInsight.tsx`
- `components/intelligence/ReviewInsight.tsx`
- `components/intelligence/MarketInsight.tsx`
- `components/intelligence/RecommendationBadge.tsx`

**Watchlist Components:**
- `components/watchlist/WatchlistCard.tsx`
- `components/watchlist/AddToWatchlistButton.tsx`
- `components/watchlist/EnableAlertsCard.tsx`
- `components/watchlist/EmptyWatchlist.tsx`
- `components/watchlist/AddToHomeScreenModal.tsx`

**Strategy Components:**
- `components/strategy/EventBanner.tsx`
- `components/strategy/AIOpinionCard.tsx`
- `components/strategy/FinalDecisionCard.tsx`
- `components/strategy/StrategyLoadingState.tsx`

**Shared:**
- `components/ShareButton.tsx`
- `components/LoadingSpinner.tsx`

---

## Build Order by Phase

### Phase 1 — Foundation (build first, everything depends on this)
1. `types/index.ts`
2. `lib/utils.ts` (cn, isIOS, isStandalone, getDeviceId)
3. `lib/api.ts` (typed fetch wrapper)
4. `hooks/useDeviceId.ts`
5. `app/layout.tsx` (PWA meta tags, font, dark theme)
6. `components/BottomNav.tsx`
7. shadcn/ui component installs (Button, Card, Badge, Sheet, Skeleton, Toast)
8. `app/page.tsx` (redirect to /scan)

### Phase 2 — Scan Flow
9. `hooks/useScanner.ts`
10. `components/scan/CameraPermissionDenied.tsx`
11. `components/scan/BarcodeScanner.tsx` (dynamic import)
12. `components/scan/ImageUpload.tsx`
13. `components/scan/ScanResult.tsx`
14. `app/scan/page.tsx`

### Phase 3 — Intelligence Card
15. `components/intelligence/RecommendationBadge.tsx`
16. `components/intelligence/IntelligenceCardSkeleton.tsx`
17. `components/intelligence/PricingInsight.tsx`
18. `components/intelligence/ReviewInsight.tsx`
19. `components/intelligence/MarketInsight.tsx`
20. `components/intelligence/IntelligenceCard.tsx`
21. `components/watchlist/AddToWatchlistButton.tsx`
22. `components/ShareButton.tsx`
23. `app/product/[id]/page.tsx`

### Phase 4 — Watchlist
24. `hooks/useWatchlist.ts`
25. `components/watchlist/EmptyWatchlist.tsx`
26. `components/watchlist/WatchlistCard.tsx`
27. `app/watchlist/page.tsx`

### Phase 5 — Push Notifications
28. `components/watchlist/AddToHomeScreenModal.tsx`
29. `components/watchlist/EnableAlertsCard.tsx`
30. `hooks/usePushNotifications.ts`
31. Update `app/watchlist/page.tsx` to include EnableAlertsCard
32. `public/manifest.json`
33. Service worker push handler (added to next-pwa config)

### Phase 6 — Strategy + Admin
34. `components/strategy/StrategyLoadingState.tsx`
35. `components/strategy/EventBanner.tsx`
36. `components/strategy/AIOpinionCard.tsx`
37. `components/strategy/FinalDecisionCard.tsx`
38. `app/strategy/[eventId]/page.tsx`
39. `app/admin/page.tsx` (minimal HTML form)

### Phase 7 — Polish
40. Animation passes on IntelligenceCard (fade-in reveal, stagger)
41. Animation on StrategyPage (sequential AI card reveals)
42. Haptic feedback on product identified (`navigator.vibrate([100, 50, 100])`)
43. Error boundaries on all pages
44. Loading states review pass

---

# 7. Backend Build Order

## Complete File List

```
backend/app/
├── main.py
├── core/
│   └── config.py
├── schemas/
│   └── api.py
├── models/
│   └── supabase_client.py
├── services/
│   ├── llm.py
│   ├── product_lookup.py
│   ├── push_service.py
│   └── event_simulator.py
├── prompts/
│   ├── intelligence.py
│   └── strategy.py
└── api/
    ├── health.py
    ├── scan.py
    ├── intelligence.py
    ├── watchlist.py
    ├── push.py
    ├── events.py
    └── strategy.py
```

## Implementation Order (Backend must stay deployable at every step)

### Step 1 — Core Infrastructure
1. `requirements.txt` — pin all dependencies
2. `core/config.py` — Settings class (pydantic-settings), reads all env vars
3. `main.py` — FastAPI app, CORS middleware, router imports, startup/shutdown
4. `api/health.py` — GET /health endpoint
5. **CHECKPOINT: Deploy to Railway. `/health` must return 200.**

### Step 2 — Schema Definitions
6. `schemas/api.py` — all Pydantic request/response models
   (ProductResponse, IntelligenceCardResponse, StrategyResponse, etc.)

### Step 3 — Database Client
7. `models/supabase_client.py` — async Supabase client singleton
8. **CHECKPOINT: Test Supabase connection from Railway logs.**

### Step 4 — LLM Service
9. `services/llm.py` — OpenRouter async client, clean_json_response(), retry logic
10. `prompts/intelligence.py` — intelligence card prompt template
11. `prompts/strategy.py` — strategy prompt template
12. **CHECKPOINT: Test LLM call via curl. Verify model IDs resolve on OpenRouter.**

### Step 5 — Product Lookup
13. `services/product_lookup.py` — Open Food Facts → UPC Item DB → return None
14. `api/scan.py` — POST /api/scan/barcode + POST /api/scan/image
15. **CHECKPOINT: Scan a real barcode. Verify product returned.**

### Step 6 — Intelligence
16. `api/intelligence.py` — POST /api/intelligence with Supabase cache check
17. **CHECKPOINT: Call intelligence endpoint. Verify AI card returned and cached.**

### Step 7 — Watchlist
18. `api/watchlist.py` — POST, GET, DELETE /api/watchlist
19. **CHECKPOINT: Add product to watchlist, retrieve, delete.**

### Step 8 — Push Notifications
20. `services/push_service.py` — VAPID config, asyncio.to_thread(webpush), 410 handling
21. `api/push.py` — POST /api/push/subscribe, DELETE /api/push/subscribe
22. **CHECKPOINT: Subscribe from browser. Verify row in Supabase.**

### Step 9 — Event Simulator + Strategy
23. `services/event_simulator.py` — build event_data from event_type, notify watchers
24. `api/events.py` — POST /api/events/simulate with BackgroundTasks
25. `api/strategy.py` — POST /api/strategy with Supabase cache check
26. **CHECKPOINT: Trigger event from /admin. Verify push received on phone.**

---

# 8. AI Integration Plan

## Architecture

All LLM interactions go through a single `services/llm.py` module.
No other file should call the OpenRouter API directly.

```
┌─────────────────────────────────────────────────────┐
│                   services/llm.py                   │
│                                                     │
│  LLMClient                                          │
│  ├── _client: httpx.AsyncClient (timeout=30s)       │
│  ├── call_text(prompt, model, response_schema)       │
│  │   ├── POST to OpenRouter                         │
│  │   ├── clean_json_response()                      │
│  │   ├── json.loads()                               │
│  │   ├── Pydantic validate against response_schema  │
│  │   └── retry(1) on validation failure             │
│  └── call_vision(image_b64, prompt, model)          │
│      └── Same flow with image in messages           │
└─────────────────────────────────────────────────────┘
```

## Model Assignment

| Task | Model | Why |
|---|---|---|
| Intelligence card | `deepseek/deepseek-chat-v4-5` (verify ID) | Strong reasoning, recommended tier |
| Strategy generation | `deepseek/deepseek-chat-v4-5` | Same model, consistent quality |
| Vision product ID | `minimax/minimax-m3` (verify ID) | Vision-capable, Champion tier |

**First action in implementation:** Call `GET https://openrouter.ai/api/v1/models` and find correct slugs for DeepSeek V4 Pro and Minimax M3.

## Prompt Management

Prompts are stored in `prompts/intelligence.py` and `prompts/strategy.py` as Python string constants.
They are NOT stored in the database — no prompt versioning system needed for MVP.
They use `.format(**kwargs)` for variable injection.

**Rules for all prompts:**
1. Must contain the word "JSON" to enable `response_format: json_object`
2. Must end with `Return ONLY valid JSON:` followed by the schema
3. Temperature: 0.3
4. Max tokens: 1500 for intelligence, 2000 for strategy

## Retry Strategy

```
Attempt 1: Call LLM
  → Success + valid JSON: return result
  → LLM API error (4xx/5xx): raise LLMError (no retry on API errors)
  → Timeout (30s): raise LLMTimeoutError (no retry)
  → JSON parse error: attempt clean_json_response(), retry once
  → Pydantic validation failure: retry once with stricter prompt addition

Attempt 2 (one retry only):
  → Same flow
  → Any failure: raise LLMError, caller returns SERVICE_UNAVAILABLE
```

## clean_json_response() Logic

```python
def clean_json_response(text: str) -> str:
    # Strip markdown code fences
    text = text.strip()
    if text.startswith("```"):
        text = text.split("```")[1]
        if text.startswith("json"):
            text = text[4:]
    if text.endswith("```"):
        text = text[:-3]
    return text.strip()
```

## JSON Validation

Every LLM response is validated against a Pydantic model before being returned.
Pydantic models are defined in `schemas/api.py`, not in the prompt module.

The validation models are strict: all fields required, correct types enforced.
If validation fails, the retry adds this to the prompt:
```
IMPORTANT: Your previous response was invalid. Make sure every field is present and every value is a non-empty string.
```

## Timeout Handling

- httpx timeout: 30 seconds total (connect + read)
- If timeout: return `*_GENERATION_FAILED` error to frontend
- Frontend shows "AI is thinking hard — try again" with a manual retry button
- No automatic frontend retry (could double credit usage)

---

# 9. Push Notification Plan

## Subscription Flow

```
User lands on /watchlist
    │
    ├─ Is push supported? (check 'Notification' in window && 'serviceWorker' in navigator)
    │   └─ No: hide EnableAlertsCard (silently)
    │
    ├─ Is iOS? (check userAgent for iPhone|iPad)
    │   └─ Yes: Is standalone? (check display-mode: standalone)
    │           └─ No: Show AddToHomeScreenModal with step-by-step guide
    │                  (Do NOT call requestPermission — it will fail)
    │
    ├─ Is permission already 'granted'? 
    │   └─ Yes: Subscribe silently and show "Alerts enabled" state
    │
    └─ Is permission 'denied'?
        └─ Yes: Show "Notifications blocked. Open Settings > Safari > Notifications"

User taps "Enable Market Alerts" button
    │
    └─ Notification.requestPermission() called IN onClick handler
        ├─ 'granted': call pushManager.subscribe({
        │      userVisibleOnly: true,
        │      applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY)
        │  })
        │  → POST /api/push/subscribe with subscription object
        │  → Update local state to "subscribed"
        │
        ├─ 'denied': show "You've blocked notifications. Reset in Settings."
        └─ 'default': show "Please allow notifications when prompted"
```

## Service Worker Push Handler

Defined in `next-pwa` custom worker configuration:

```javascript
// Handles incoming push messages
self.addEventListener('push', (event) => {
  if (!event.data) return;
  const { title, body, url, icon } = event.data.json();
  
  event.waitUntil(
    self.registration.showNotification(title, {
      body,
      icon: icon || '/icons/icon-192.png',
      badge: '/icons/badge-72.png',
      vibrate: [100, 50, 100],
      tag: 'bazaarlens-alert',    // replaces previous notification (prevents spam)
      renotify: true,
      data: { url }
    })
  );
});

// Handles notification click — opens strategy page
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url || '/watchlist';
  
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true })
      .then((clientList) => {
        // If app is already open, focus it
        for (const client of clientList) {
          if (client.url.includes(self.location.origin) && 'focus' in client) {
            client.focus();
            client.navigate(url);
            return;
          }
        }
        // Otherwise open new window
        return clients.openWindow(url);
      })
  );
});
```

## Backend Notification Flow

```python
# services/push_service.py

async def send_notification(subscription_row: dict, payload: dict) -> bool:
    """
    Send a push notification to one subscription.
    Returns True on success, False on expired subscription.
    Raises on other errors.
    """
    try:
        await asyncio.to_thread(
            webpush,
            subscription_info={
                "endpoint": subscription_row["endpoint"],
                "keys": {
                    "p256dh": subscription_row["p256dh"],
                    "auth": subscription_row["auth_key"]
                }
            },
            data=json.dumps(payload),
            vapid_private_key=settings.VAPID_PRIVATE_KEY,
            vapid_claims={"sub": settings.VAPID_EMAIL}
        )
        return True
    except WebPushException as e:
        if e.response and e.response.status_code == 410:
            # Subscription expired — delete it
            await supabase.table("push_subscriptions")
                .delete()
                .eq("device_id", subscription_row["device_id"])
                .execute()
            return False
        raise
```

## Push Notification Payload Format

```json
{
  "title": "Market Alert: Competitor Price Drop",
  "body": "A competitor dropped price by 20% on your watched product. See strategy →",
  "url": "/strategy/880e8400-e29b-41d4-a716-446655440000",
  "icon": "/icons/icon-192.png"
}
```

## Testing Strategy

**Phase 1 — Desktop Chrome (fastest iteration):**
- Use Chrome DevTools → Application → Service Workers to inspect SW
- Use Chrome DevTools → Application → Push to send manual test push
- No real VAPID needed at this stage — use Chrome's test push

**Phase 2 — Real device, real push:**
- Deploy to Vercel (HTTPS required)
- Open on iPhone/Android
- Go through full subscription flow
- Test from `/admin` page on laptop

**Phase 3 — Demo rehearsal:**
- Full end-to-end: scan → watchlist → admin trigger → push received → strategy page
- Test 3 times in a row
- Test with phone screen cast to monitor/TV (Office Kit screen mirror equivalent)

**Edge Cases to Test:**
- Subscription expiry simulation: manually delete push_subscription row from Supabase; trigger event; verify no crash
- Double subscription: subscribe twice from same device; verify single row (upsert works)
- Notification tap when app is backgrounded vs closed
- iOS: notification tap when PWA is installed vs not installed

---

# 10. Final Milestone Roadmap

## Time Budget

Total: 30 hours | Buffer: 4 hours | Build time: 26 hours

---

### Milestone 1 — Project Bootstrap
**Goal:** Both projects initialize, build, and connect to each other.
**Duration:** 2 hours

**Files Affected:**
- `frontend/` — Next.js init, tailwind, shadcn/ui, `@ducanh2912/next-pwa`
- `backend/` — FastAPI init, requirements.txt, main.py
- `frontend/.env.local`, `backend/.env`

**Acceptance Criteria:**
- [ ] `npm run dev` starts frontend at localhost:3000 with no errors
- [ ] `uvicorn app.main:app --reload` starts backend at localhost:8000
- [ ] GET `localhost:8000/health` returns `{"status": "ok"}`
- [ ] Frontend can call backend `/health` and render the response

---

### Milestone 2 — Supabase + Deployment
**Goal:** Schema in Supabase, both services deployed and reachable.
**Duration:** 2 hours

**Files Affected:**
- Supabase SQL editor (run migration from docs/blueprint.md)
- Vercel project created, env vars set
- Railway project created, env vars set
- `backend/core/config.py`
- `backend/models/supabase_client.py`

**Acceptance Criteria:**
- [ ] All 5 tables exist in Supabase with correct columns + indexes
- [ ] RLS enabled
- [ ] Backend can INSERT and SELECT from Supabase
- [ ] Frontend deployed to `*.vercel.app`, backend deployed to `*.railway.app`
- [ ] `GET https://bazaarlens-api.railway.app/health` returns 200

---

### Milestone 3 — Barcode Scanner (Frontend)
**Goal:** Camera opens, barcode is detected, result appears.
**Duration:** 2 hours

**Files Affected:**
- `types/index.ts`
- `lib/utils.ts`
- `hooks/useDeviceId.ts`
- `hooks/useScanner.ts`
- `components/scan/BarcodeScanner.tsx`
- `components/scan/CameraPermissionDenied.tsx`
- `app/scan/page.tsx`
- `app/layout.tsx`

**Acceptance Criteria:**
- [ ] Camera opens on /scan with scanning overlay
- [ ] Barcode detected within 3 seconds of pointing at a product
- [ ] Detected barcode value shown on screen
- [ ] Camera permission denied shows fallback state
- [ ] Works on both iPhone Safari and Android Chrome

---

### Milestone 4 — Product Identification (Backend + Integration)
**Goal:** Scanned barcode resolves to a real product.
**Duration:** 2 hours

**Files Affected:**
- `backend/schemas/api.py`
- `backend/services/product_lookup.py`
- `backend/api/scan.py`
- `frontend/lib/api.ts`
- `frontend/components/scan/ScanResult.tsx`

**Acceptance Criteria:**
- [ ] Scanning a Maggi barcode returns name: "Maggi 2-Minute Noodles Masala"
- [ ] Product image, brand, category displayed in ScanResult bottom sheet
- [ ] Cache hit on second scan (no Open Food Facts call, faster response)
- [ ] "Unknown product" shown gracefully if barcode not found
- [ ] `source` field distinguishes open_food_facts vs upc_db vs vision_llm

---

### Milestone 5 — Image Upload + Vision Fallback
**Goal:** Unknown products can be identified via photo.
**Duration:** 1.5 hours

**Files Affected:**
- `backend/services/llm.py` (vision call)
- `backend/api/scan.py` (POST /api/scan/image)
- `frontend/components/scan/ImageUpload.tsx`
- `frontend/services/productLookup.ts` (image resize + base64)

**Acceptance Criteria:**
- [ ] Image picker opens on tap
- [ ] Image resized to max 800px before upload
- [ ] Minimax M3 identifies a product from a clear photo
- [ ] "AI identified" badge shown when source is vision_llm
- [ ] Low confidence (<0.7) shown with warning

---

### Milestone 6 — Intelligence Card (AI + UI)
**Goal:** Full intelligence card with 4 insights appears after scan.
**Duration:** 2 hours

**Files Affected:**
- `backend/prompts/intelligence.py`
- `backend/api/intelligence.py`
- `frontend/components/intelligence/` (all components)
- `frontend/app/product/[id]/page.tsx`

**Acceptance Criteria:**
- [ ] Skeleton shows while AI generates (3–8s acceptable)
- [ ] All 4 insight panels populated with real AI content
- [ ] Recommendation badge correct color and level
- [ ] Card caches correctly — second visit for same product is instant
- [ ] Works offline for cached products (SW caches API response)

---

### Milestone 7 — Watchlist
**Goal:** Products can be added, viewed, and removed from watchlist.
**Duration:** 1.5 hours

**Files Affected:**
- `backend/api/watchlist.py`
- `frontend/hooks/useWatchlist.ts`
- `frontend/components/watchlist/WatchlistCard.tsx`
- `frontend/components/watchlist/EmptyWatchlist.tsx`
- `frontend/app/watchlist/page.tsx`
- `frontend/components/BottomNav.tsx`

**Acceptance Criteria:**
- [ ] Tap "Add to Watchlist" → product appears in /watchlist
- [ ] Remove from watchlist works
- [ ] device_id persists across page refreshes (localStorage)
- [ ] Empty watchlist state shown with CTA
- [ ] Bottom nav shows Scan / Watchlist tabs

---

### Milestone 8 — Push Notifications (Frontend)
**Goal:** Push subscription works. Test notification can be received.
**Duration:** 2 hours

**Files Affected:**
- `frontend/public/manifest.json`
- `frontend/hooks/usePushNotifications.ts`
- `frontend/components/watchlist/AddToHomeScreenModal.tsx`
- `frontend/components/watchlist/EnableAlertsCard.tsx`
- Service worker push handler (next-pwa config)
- `frontend/app/watchlist/page.tsx` (integrate EnableAlertsCard)

**Acceptance Criteria:**
- [ ] manifest.json has `display: "standalone"` and all required fields
- [ ] "Enable Market Alerts" button appears on /watchlist
- [ ] On iOS non-standalone: AddToHomeScreen modal appears instead
- [ ] On iOS standalone or Chrome: permission prompt shown on button tap
- [ ] Subscription stored in Supabase push_subscriptions
- [ ] Service worker handles push events (test via Chrome DevTools push)

---

### Milestone 9 — Push Notifications (Backend)
**Goal:** Backend can deliver real push notifications to the subscribed device.
**Duration:** 1.5 hours

**Files Affected:**
- `backend/services/push_service.py`
- `backend/api/push.py`
- VAPID keys added to Railway env vars

**Acceptance Criteria:**
- [ ] POST /api/push/subscribe saves subscription to Supabase
- [ ] Manual push (call send_notification directly from Python shell) delivers to device
- [ ] 410 handling: expired subscription deleted from DB without crashing
- [ ] Real push notification appears on iOS/Android device from backend call

---

### Milestone 10 — Market Event Simulator
**Goal:** Triggering an event from /admin sends push notification to the device.
**Duration:** 1.5 hours

**Files Affected:**
- `backend/services/event_simulator.py`
- `backend/api/events.py`
- `frontend/app/admin/page.tsx`

**Acceptance Criteria:**
- [ ] /admin?key=demo2026 loads a form with product dropdown + event type
- [ ] Triggering "Competitor Price Drop" → instant 200 response
- [ ] Background task: strategy generated, push sent
- [ ] Push notification received on demo device within 15 seconds
- [ ] market_events row written with notification_sent=true

---

### Milestone 11 — Strategy Page
**Goal:** Tapping notification opens a complete strategy page.
**Duration:** 2 hours

**Files Affected:**
- `backend/prompts/strategy.py`
- `backend/api/strategy.py`
- `frontend/components/strategy/` (all components)
- `frontend/app/strategy/[eventId]/page.tsx`

**Acceptance Criteria:**
- [ ] Notification tap opens /strategy/[eventId]
- [ ] EventBanner shows event type and summary
- [ ] All 3 AI opinion cards populated (marketing, product, sales)
- [ ] Final decision card highlighted with urgency
- [ ] "Share Report" button triggers navigator.share()
- [ ] Strategy is cached — second visit is instant

---

### Milestone 12 — Polish Pass
**Goal:** The app looks and feels premium. Demo is smooth.
**Duration:** 2 hours

**Files Affected:** All components (animation, spacing, color, typography review)

**Acceptance Criteria:**
- [ ] Intelligence card has staggered fade-in reveal animation
- [ ] Strategy page reveals 3 AI cards sequentially (0.3s stagger)
- [ ] Haptic on product identified (`navigator.vibrate([100, 50, 100])`)
- [ ] All loading skeletons look polished
- [ ] No console errors in production build
- [ ] Lighthouse PWA score ≥ 90

---

### Milestone 13 — Demo Rehearsal
**Goal:** Full demo works flawlessly 3 times in a row.
**Duration:** 2 hours

**Process:**
1. Install PWA to demo device home screen
2. Grant push notification permission
3. Run full demo flow (scan → intelligence → watchlist → admin trigger → push → strategy)
4. Fix anything that breaks
5. Repeat 3 times without a restart
6. Prepare 2.5-minute verbal script (see docs/demo-flow.md in CLAUDE.md)

**Acceptance Criteria:**
- [ ] Demo completed in < 2.5 minutes
- [ ] No manual fallbacks needed
- [ ] Push notification arrives within 10 seconds of admin trigger
- [ ] Strategy page loads in < 3 seconds (cached)
- [ ] Share button works and shows iQOO share sheet

---

## Milestone Dependency Chart

```
M1 (Bootstrap)
    │
    ├── M2 (Supabase + Deploy)
    │       │
    │       ├── M3 (Scanner Frontend)
    │       │       │
    │       │       └── M4 (Product ID) ──── M5 (Vision Fallback)
    │       │               │
    │       │               └── M6 (Intelligence Card)
    │       │                       │
    │       │                       └── M7 (Watchlist)
    │       │                               │
    │       │                ┌──────────────┤
    │       │                │              │
    │       │           M8 (Push FE)   M10 (Event Sim)
    │       │                │              │
    │       │           M9 (Push BE)        │
    │       │                └──────────────┤
    │       │                               │
    │       │                         M11 (Strategy)
    │       │                               │
    │       │                         M12 (Polish)
    │       │                               │
    │       │                         M13 (Demo Rehearsal)
```

---

## Time Allocation Summary

| Milestone | Hours |
|---|---|
| M1 — Bootstrap | 2.0 |
| M2 — Supabase + Deploy | 2.0 |
| M3 — Scanner Frontend | 2.0 |
| M4 — Product Identification | 2.0 |
| M5 — Vision Fallback | 1.5 |
| M6 — Intelligence Card | 2.0 |
| M7 — Watchlist | 1.5 |
| M8 — Push Frontend | 2.0 |
| M9 — Push Backend | 1.5 |
| M10 — Event Simulator | 1.5 |
| M11 — Strategy Page | 2.0 |
| M12 — Polish | 2.0 |
| M13 — Demo Rehearsal | 2.0 |
| **Buffer** | **4.0** |
| **TOTAL** | **30.0** |

---

*Blueprint version 1.0 — 2026-06-13*
*All decisions from CLAUDE.md have been validated and expanded here.*
*Next step: begin Milestone 1.*
