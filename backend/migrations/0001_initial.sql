-- products: cache of identified products
CREATE TABLE IF NOT EXISTS products (
  id          TEXT PRIMARY KEY,           -- UUID v4 as text
  barcode     TEXT UNIQUE,
  name        TEXT NOT NULL,
  brand       TEXT,
  category    TEXT,
  image_url   TEXT,
  source      TEXT NOT NULL DEFAULT 'unknown',
  raw_data    TEXT,                        -- JSON stored as text in SQLite
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- intelligence_cards: AI-generated insights per product (1:1)
CREATE TABLE IF NOT EXISTS intelligence_cards (
  id                   TEXT PRIMARY KEY,
  product_id           TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  pricing_insight      TEXT NOT NULL,
  review_insight       TEXT NOT NULL,
  market_insight       TEXT NOT NULL,
  recommendation       TEXT NOT NULL,
  recommendation_level TEXT NOT NULL CHECK (recommendation_level IN ('buy','hold','avoid','watch')),
  confidence           REAL,
  model_used           TEXT,
  generated_at         TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(product_id)
);

-- watchlist: device-keyed product monitoring (no auth)
CREATE TABLE IF NOT EXISTS watchlist (
  id          TEXT PRIMARY KEY,
  device_id   TEXT NOT NULL,
  product_id  TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  added_at    TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(device_id, product_id)
);

-- push_subscriptions: VAPID Web Push subscriptions per device
CREATE TABLE IF NOT EXISTS push_subscriptions (
  id          TEXT PRIMARY KEY,
  device_id   TEXT NOT NULL UNIQUE,
  endpoint    TEXT NOT NULL,
  p256dh      TEXT NOT NULL,
  auth_key    TEXT NOT NULL,
  is_active   INTEGER NOT NULL DEFAULT 1,
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- market_events: simulated market events (demo trigger)
CREATE TABLE IF NOT EXISTS market_events (
  id                  TEXT PRIMARY KEY,
  product_id          TEXT NOT NULL REFERENCES products(id) ON DELETE CASCADE,
  event_type          TEXT NOT NULL CHECK (event_type IN ('price_drop','demand_spike','sentiment_crash','stock_outage')),
  event_data          TEXT NOT NULL DEFAULT '{}',
  notification_sent   INTEGER NOT NULL DEFAULT 0,
  notifications_count INTEGER NOT NULL DEFAULT 0,
  triggered_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

-- strategies: AI-generated strategies per event (cached)
CREATE TABLE IF NOT EXISTS strategies (
  id             TEXT PRIMARY KEY,
  event_id       TEXT NOT NULL UNIQUE REFERENCES market_events(id) ON DELETE CASCADE,
  marketing_ai   TEXT NOT NULL,    -- JSON
  product_ai     TEXT NOT NULL,    -- JSON
  sales_ai       TEXT NOT NULL,    -- JSON
  final_decision TEXT NOT NULL,    -- JSON
  model_used     TEXT,
  generated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_watchlist_device_id ON watchlist(device_id);
CREATE INDEX IF NOT EXISTS idx_watchlist_product_id ON watchlist(product_id);
CREATE INDEX IF NOT EXISTS idx_market_events_product_id ON market_events(product_id);
