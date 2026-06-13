thub # BazaarLens — CLAUDE.md

Project memory and architecture reference for all development sessions.
This file is the single source of truth. Keep it updated as decisions evolve.

---

## 1. Product Vision

BazaarLens turns a smartphone into an AI market intelligence assistant for small
business owners — Amazon sellers, Flipkart sellers, resellers, local shop owners.

The product must feel like an AI business advisor, not a dashboard or chatbot.
Users want answers to "What should I do?" not tables of data.

**Tagline:** "Point your camera at any product. Your AI intelligence team watches its market forever."

---

## 2. Hackathon Context

- **Event:** iQOO Hackathon 2026 · Bengaluru  
- **Duration:** 30 hours  
- **Prize Pool:** INR 3.2L  
- **Judging Weights:**
  - 25% — iQOO Office Kit Usage
  - 25% — Phone-First Experience
  - 20% — AI-Native Functionality
  - 20% — Real-World Utility
  - 10% — Craft, Polish, Demo Quality

---

## 3. iQOO Office Kit Strategy

**What Office Kit is:** A cross-device productivity suite in OriginOS 6.
Features include: Screen Mirroring, Multi-Device Sync (Origin Notes), Task Handoff, File EasyShare.

**There is no developer SDK.** We score the 25% via design choices:

1. **Web Share API** — "Share Intelligence Report" button uses `navigator.share()`.
   On iQOO this routes into Origin Notes, WhatsApp, etc. Native feel.
2. **Camera-first UX** — Every flow starts with the camera. Showcases iQOO hardware.
3. **Screen Mirror Demo** — During demo, mirror iQOO phone to projector/monitor.
   Show the full product scan + strategy on screen. This is the demo strategy.
4. **Responsive Design at tablet widths** — Office Kit screen mirroring can show
   the app on a larger screen. Make sure 768px+ still looks great.
5. **Offline capability** — Service worker caches the watchlist. Demonstrates the
   PWA working in edge conditions, relevant to "office productivity" context.

Score this criterion by mentioning Office Kit integration in the pitch deck.

---

## 4. MVP Scope — NON-NEGOTIABLE

Build only these 8 features, in this order:

1. Product Scan (barcode + image upload + camera)
2. Product Identification (lookup + LLM fallback)
3. Product Intelligence Card
4. Watchlist
5. Strategy Generation
6. Market Event Simulator
7. Push Notifications
8. Strategy Page

Do not add anything else until all 8 are working end-to-end.

---

## 5. Tech Stack

### Frontend
- **Framework:** Next.js 14 (App Router)
- **Language:** TypeScript
- **Styling:** TailwindCSS + shadcn/ui
- **PWA:** next-pwa (workbox)
- **Barcode Scanning:** `@zxing/browser` (not BarcodeDetector API — see Risk #1)
- **HTTP Client:** fetch (native) / SWR for data fetching
- **Push:** Web Push API + VAPID (client-side subscription)

### Backend
- **Framework:** FastAPI (Python 3.11+)
- **HTTP Client:** httpx (async)
- **Push Notifications:** `pywebpush` library
- **Env Management:** python-dotenv

### Database
- **Service:** Supabase (PostgreSQL)
- **Auth:** Supabase anonymous auth (no user accounts needed for MVP)
- **Realtime:** Supabase realtime for watchlist events (optional, use for demo effect)

### AI / LLM
Via OpenRouter API (base URL: `https://openrouter.ai/api/v1`):

| Purpose | Model | Tier |
|---|---|---|
| Product identification (text/barcode) | `deepseek/deepseek-chat-v4` | Recommended (free) |
| Intelligence card generation | `deepseek/deepseek-chat-v4` | Recommended (free) |
| Strategy generation | `deepseek/deepseek-prover-v2` | Recommended (free) |
| Vision fallback (image scan) | `minimax/minimax-m3` | Recommended (free, Champion) |

**IMPORTANT:** Gemini and Claude are Credit Intensive on the provided OpenRouter credits.
Use them only as last resort. Primary stack is DeepSeek V4 + Minimax M3.

Always use `response_format: { type: "json_object" }` for structured output.

### Product Lookup (no LLM needed for barcodes)
- **Primary:** Open Food Facts API (`https://world.openfoodfacts.org/api/v2/product/{barcode}.json`) — free, no key needed
- **Secondary:** UPC Item DB (`https://api.upcitemdb.com/prod/trial/lookup?upc={barcode}`) — free tier
- **Fallback:** Minimax M3 vision if both fail and image is available

### Deployment
- **Frontend:** Vercel (`bazaarlens.vercel.app` or custom domain)
- **Backend:** Railway or Render (FastAPI container)
- **Database:** Supabase cloud (free tier)

---

## 6. Folder Structure

```
bazaarlens/
├── frontend/
│   ├── app/
│   │   ├── layout.tsx              # Root layout, PWA meta tags
│   │   ├── page.tsx                # Home → redirect to /scan
│   │   ├── scan/
│   │   │   └── page.tsx            # Scan page
│   │   ├── product/
│   │   │   └── [id]/
│   │   │       └── page.tsx        # Product intelligence card
│   │   ├── watchlist/
│   │   │   └── page.tsx            # Watchlist page
│   │   ├── strategy/
│   │   │   └── [eventId]/
│   │   │       └── page.tsx        # Strategy page
│   │   └── api/
│   │       └── push/
│   │           └── route.ts        # Push subscription endpoint (Next.js route)
│   ├── components/
│   │   ├── scan/
│   │   │   ├── BarcodeScanner.tsx  # Camera + ZXing scanner
│   │   │   ├── ImageUpload.tsx     # File picker fallback
│   │   │   └── ScanResult.tsx      # Shows identified product
│   │   ├── intelligence/
│   │   │   ├── IntelligenceCard.tsx
│   │   │   ├── PricingInsight.tsx
│   │   │   ├── ReviewInsight.tsx
│   │   │   └── RecommendationBadge.tsx
│   │   ├── strategy/
│   │   │   ├── StrategyPage.tsx
│   │   │   ├── AIOpinionCard.tsx   # marketing/product/sales opinions
│   │   │   └── FinalDecision.tsx
│   │   ├── watchlist/
│   │   │   ├── WatchlistCard.tsx
│   │   │   └── AddToWatchlist.tsx
│   │   └── ui/                     # shadcn/ui components live here
│   ├── hooks/
│   │   ├── useScanner.ts           # ZXing scanning hook
│   │   ├── usePushNotifications.ts # Push subscription hook
│   │   └── useWatchlist.ts         # Watchlist CRUD hook
│   ├── lib/
│   │   ├── supabase.ts             # Supabase client
│   │   ├── api.ts                  # Backend API client
│   │   └── utils.ts                # cn(), formatters
│   ├── services/
│   │   └── productLookup.ts        # Open Food Facts + UPC lookup
│   ├── types/
│   │   └── index.ts                # All TypeScript types
│   └── public/
│       ├── manifest.json
│       ├── sw.js                   # Service worker (generated by next-pwa)
│       └── icons/                  # PWA icons (192, 512, maskable)
│
├── backend/
│   ├── app/
│   │   ├── main.py                 # FastAPI app, CORS, router includes
│   │   ├── api/
│   │   │   ├── scan.py             # POST /scan (vision LLM)
│   │   │   ├── intelligence.py     # POST /intelligence
│   │   │   ├── strategy.py         # POST /strategy
│   │   │   ├── watchlist.py        # GET/POST /watchlist
│   │   │   ├── events.py           # POST /events/simulate (demo trigger)
│   │   │   └── push.py             # POST /push/subscribe, POST /push/send
│   │   ├── services/
│   │   │   ├── llm.py              # OpenRouter API client
│   │   │   ├── product_lookup.py   # Open Food Facts + UPC lookup
│   │   │   ├── push_service.py     # pywebpush VAPID logic
│   │   │   └── event_simulator.py  # Market event generation
│   │   ├── prompts/
│   │   │   ├── intelligence.py     # Intelligence card prompt template
│   │   │   └── strategy.py         # Multi-perspective strategy prompt
│   │   ├── models/
│   │   │   └── supabase_client.py  # Supabase Python client
│   │   ├── schemas/
│   │   │   └── api.py              # Pydantic request/response models
│   │   └── core/
│   │       └── config.py           # Settings (pydantic-settings)
│   ├── requirements.txt
│   └── tests/
│
├── docs/
│   ├── api.md                      # API endpoint reference
│   └── demo-flow.md                # Step-by-step demo script
│
├── CLAUDE.md                       # This file
└── README.md
```

---

## 7. Database Schema (Supabase)

```sql
-- Products cache (avoid re-fetching)
CREATE TABLE products (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  barcode TEXT UNIQUE,
  name TEXT NOT NULL,
  brand TEXT,
  category TEXT,
  image_url TEXT,
  raw_data JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Intelligence cards (cached per product)
CREATE TABLE intelligence_cards (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID REFERENCES products(id),
  pricing_insight TEXT,
  review_insight TEXT,
  market_insight TEXT,
  recommendation TEXT,
  recommendation_level TEXT CHECK (recommendation_level IN ('buy', 'hold', 'avoid', 'watch')),
  generated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Watchlist (anonymous user sessions via device_id)
CREATE TABLE watchlist (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  device_id TEXT NOT NULL,        -- localStorage UUID, no login needed
  product_id UUID REFERENCES products(id),
  added_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(device_id, product_id)
);

-- Push subscriptions (linked to device_id)
CREATE TABLE push_subscriptions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  device_id TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  p256dh TEXT NOT NULL,
  auth TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(device_id)
);

-- Market events (for simulator + notifications)
CREATE TABLE market_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  product_id UUID REFERENCES products(id),
  event_type TEXT CHECK (event_type IN ('price_drop', 'demand_spike', 'sentiment_crash', 'stock_outage')),
  event_data JSONB,
  triggered_at TIMESTAMPTZ DEFAULT NOW()
);

-- Strategy outputs (cached)
CREATE TABLE strategies (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id UUID REFERENCES market_events(id),
  marketing_ai JSONB,
  product_ai JSONB,
  sales_ai JSONB,
  final_decision JSONB,
  generated_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 8. API Design

### Backend Base URL: `https://bazaarlens-api.railway.app`

```
POST /api/scan/barcode       { barcode: string } → product
POST /api/scan/image         { image_base64: string } → product (vision LLM)
POST /api/intelligence       { product_id: string } → intelligence card
POST /api/watchlist          { device_id, product_id } → add to watch
GET  /api/watchlist/{device_id}  → list watched products
POST /api/events/simulate    { product_id, event_type } → triggers notification
POST /api/strategy           { event_id: string } → strategy output
POST /api/push/subscribe     { device_id, subscription: PushSubscription } → save
POST /api/push/send          { device_id, payload } → send push (internal)
```

All responses follow:
```json
{ "success": true, "data": {...}, "error": null }
```

---

## 9. AI / LLM Prompting Strategy

### Core Principle
One prompt, one LLM call, structured JSON output. No agents, no orchestration.

### Intelligence Card Prompt Pattern
```python
INTELLIGENCE_PROMPT = """
You are a market intelligence analyst for small business owners in India.

Product: {product_name}
Category: {category}
Available Data: {market_context}

Return ONLY valid JSON:
{{
  "pricing_insight": "1-2 sentence insight on pricing opportunity",
  "review_insight": "1-2 sentence insight on customer sentiment",
  "market_insight": "1-2 sentence insight on market position",
  "recommendation": "specific actionable recommendation",
  "recommendation_level": "buy|hold|avoid|watch",
  "confidence": 0.0-1.0
}}
"""
```

### Strategy Prompt Pattern (Multi-perspective in one call)
```python
STRATEGY_PROMPT = """
You are a business strategy AI. A market event has occurred.

Product: {product_name}
Event: {event_type} — {event_description}
Market Context: {context}

Respond as three AI advisors, then give a final unified strategy.
Return ONLY valid JSON:
{{
  "marketing_ai": {{
    "opinion": "marketing perspective",
    "action": "specific marketing action",
    "urgency": "high|medium|low"
  }},
  "product_ai": {{
    "opinion": "product perspective",
    "action": "specific product action",
    "urgency": "high|medium|low"
  }},
  "sales_ai": {{
    "opinion": "sales perspective",
    "action": "specific sales action",
    "urgency": "high|medium|low"
  }},
  "final_decision": {{
    "recommended_action": "the single most important action to take now",
    "reasoning": "why this is the right move",
    "risk_assessment": "main risks if action is not taken",
    "time_sensitivity": "act within X hours/days"
  }}
}}
"""
```

### LLM Client Pattern (OpenRouter)
```python
async def call_llm(prompt: str, model: str = "deepseek/deepseek-chat-v4") -> dict:
    response = await httpx_client.post(
        "https://openrouter.ai/api/v1/chat/completions",
        headers={"Authorization": f"Bearer {OPENROUTER_API_KEY}"},
        json={
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "response_format": {"type": "json_object"},
            "temperature": 0.3,
        }
    )
    return json.loads(response.json()["choices"][0]["message"]["content"])
```

---

## 10. Coding Standards

### TypeScript / Frontend
- Use `function` keyword for components (not arrow functions for page-level)
- All API types defined in `types/index.ts`
- No `any`. Use `unknown` and narrow.
- Import shadcn/ui components from `@/components/ui/`
- Tailwind only — no inline styles, no CSS modules
- Mobile-first breakpoints: design for 390px width first
- Every page should work at 390px width (iPhone 14 Pro) and 414px (iQOO reference)

### Python / Backend
- Pydantic models for all request/response schemas
- `async def` for all route handlers
- No print statements — use Python logging
- Environment vars via `pydantic-settings` Settings class
- `httpx.AsyncClient` for all external HTTP calls (not requests)
- All LLM calls wrapped in try/except with graceful fallback

### General
- No commented-out code in commits
- No TODO comments — create GitHub issue instead
- No console.log in production code
- Keep each file under 200 lines — extract when it grows

---

## 11. UI Principles

### Visual Identity
- **Dark mode only** — matches iQOO's gaming/premium aesthetic
- **Primary color:** Electric blue (`#2563EB` / blue-600)
- **Accent:** Amber for alerts/opportunities (`#F59E0B`)
- **Background:** Near-black (`#0A0A0A` / zinc-950)
- **Card background:** `#111111` / zinc-900
- **Text:** White primary, zinc-400 secondary

### Mobile-First Rules
1. Touch targets minimum 44×44px (Apple HIG)
2. Bottom navigation bar (thumb-reachable)
3. No hover-dependent interactions
4. Camera fills full viewport during scan
5. Cards use large, readable text (minimum 16px body)
6. Key actions use full-width buttons at page bottom

### Motion
- Subtle fade-in on intelligence card reveal (feel like "AI thinking")
- Pulse animation on recommendation badge
- Success haptic feedback on product identified (if `navigator.vibrate` available)

### Scan UX
- Camera opens full-screen immediately
- Scanning overlay with animated corners
- "Add to Watchlist" bottom sheet slides up after identification

---

## 12. Push Notifications Setup

### Architecture
- Backend generates VAPID key pair (once, store in env)
- Frontend uses `navigator.serviceWorker` + `pushManager.subscribe()`
- Subscription endpoint + keys stored in Supabase `push_subscriptions`
- Backend calls `pywebpush` to deliver notification

### iOS Requirements (CRITICAL FOR DEMO)
1. Site must be served over HTTPS
2. `manifest.json` must have `"display": "standalone"`
3. PWA must be installed to home screen via Safari → Share → Add to Home Screen
4. Permission must be requested via user gesture (button tap)
5. Requires iOS 16.4+

### Demo Preparation
- Install PWA on demo device BEFORE the presentation
- Grant notification permission BEFORE the presentation
- Test notification delivery on WiFi AND cellular
- Have a second device as backup

### Service Worker (public/sw.js — generated by next-pwa)
The service worker must handle `push` events and display notifications:
```js
self.addEventListener('push', event => {
  const data = event.data?.json();
  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: '/icons/icon-192.png',
      badge: '/icons/badge-72.png',
      data: { url: data.url }
    })
  );
});
self.addEventListener('notificationclick', event => {
  event.notification.close();
  event.waitUntil(clients.openWindow(event.notification.data.url));
});
```

---

## 13. Market Event Simulator

**This is the most important feature for the demo.**

### Available Event Types
| Type | Display | What it means |
|---|---|---|
| `price_drop` | Competitor Price Drop | A competitor dropped price by X% |
| `demand_spike` | Demand Spike | Search volume spiked 3x overnight |
| `sentiment_crash` | Sentiment Crash | Reviews dropped from 4.2 to 2.8 stars |
| `stock_outage` | Stock Outage | Top competitor is out of stock |

### Trigger Mechanism
- Hidden admin panel at `/admin` (no auth, just obscure)
- Simple UI: select product from watchlist + select event type → trigger
- Backend: `POST /api/events/simulate` → generates event → sends push notification → stores event

### Demo Script
1. Show product scan on phone
2. Add product to watchlist
3. On laptop/second device: open `/admin`, trigger "Competitor Price Drop"
4. Demo device receives push notification
5. Tap notification → opens Strategy page
6. Strategy page shows 3 AI opinions + final decision

---

## 14. Anonymous Auth Strategy

No user login required. Authentication-free UX for faster onboarding.

- Generate `device_id` = `crypto.randomUUID()` on first visit
- Store in `localStorage` under key `bazaarlens_device_id`
- All API calls pass `device_id` as header `X-Device-Id`
- Watchlist and push subscriptions keyed by `device_id`
- **Implication:** If user clears browser data, they lose their watchlist (acceptable for MVP)

---

## 15. Phase Roadmap

### Phase 1 — Foundation (Hours 1-4)
- [ ] Initialize Next.js project with TypeScript + Tailwind + shadcn/ui
- [ ] Initialize FastAPI backend with project structure
- [ ] Set up Supabase project + run schema migrations
- [ ] Create environment variables for both projects
- [ ] Deploy frontend skeleton to Vercel, backend to Railway
- [ ] Verify end-to-end connectivity

### Phase 2 — Scan + Identify (Hours 4-10)
- [ ] Implement `BarcodeScanner.tsx` with ZXing
- [ ] Implement `ImageUpload.tsx` fallback
- [ ] Implement Open Food Facts lookup in backend
- [ ] Implement vision LLM fallback (Minimax M3) for unknown products
- [ ] Build `ScanResult.tsx` with identified product preview
- [ ] Add "Add to Watchlist" action

### Phase 3 — Intelligence Card (Hours 10-14)
- [ ] Implement intelligence generation prompt
- [ ] Build `IntelligenceCard.tsx` (full-screen card with all 4 insights)
- [ ] Add recommendation badge with color coding
- [ ] Cache intelligence in Supabase to avoid re-generation

### Phase 4 — Watchlist (Hours 14-16)
- [ ] Implement watchlist API routes
- [ ] Build `WatchlistPage` with product cards
- [ ] Add remove from watchlist action

### Phase 5 — Push Notifications (Hours 16-20)
- [ ] Generate VAPID keys, add to backend env
- [ ] Implement push subscription endpoint
- [ ] Build `usePushNotifications` hook
- [ ] Implement "Enable Alerts" button with iOS instructions modal
- [ ] Test push on real device

### Phase 6 — Event Simulator + Strategy (Hours 20-26)
- [ ] Build Market Event Simulator in backend
- [ ] Build `/admin` trigger page (minimal UI)
- [ ] Implement strategy generation prompt
- [ ] Build `StrategyPage` with 3 AI cards + final decision

### Phase 7 — Polish + Demo Prep (Hours 26-30)
- [ ] Polish animations and transitions
- [ ] PWA manifest + icons
- [ ] Share button (Web Share API) 
- [ ] Full demo run-through × 3
- [ ] Fix anything that breaks during dry run
- [ ] Prepare pitch talking points

---

## 16. Demo Flow (Rehearse This Exactly)

```
[PRESENTER HAS iQOO PHONE WITH PWA PRE-INSTALLED AND PUSH ENABLED]

0:00  Open BazaarLens on iQOO phone (cast to screen via Office Kit mirroring)
0:20  Tap scan → camera opens → hold up product (physical product in hand!)
0:35  Product identified — name appears with confidence
0:45  "See Intelligence" → card slides up with 4 insights + recommendation
1:15  "Add to Watchlist" → product added
1:25  Switch to laptop → open /admin → select product → "Competitor Price Drop"
1:30  Phone receives push notification (audience sees it live!)
1:35  Tap notification → strategy page opens
2:00  Walk through 3 AI perspectives (marketing, product, sales)
2:20  Show final decision: "Lower price by 8%, run flash sale within 24 hours"
2:30  Show "Share Report" → Web Share sheet opens (iQOO Office Kit moment!)
2:40  Done — hand off to Q&A

TOTAL DEMO TIME: ~2.5 minutes
```

---

## 17. Environment Variables

### Frontend (.env.local)
```
NEXT_PUBLIC_API_URL=https://bazaarlens-api.railway.app
NEXT_PUBLIC_SUPABASE_URL=...
NEXT_PUBLIC_SUPABASE_ANON_KEY=...
NEXT_PUBLIC_VAPID_PUBLIC_KEY=...
```

### Backend (.env)
```
OPENROUTER_API_KEY=...
SUPABASE_URL=...
SUPABASE_SERVICE_KEY=...
VAPID_PRIVATE_KEY=...
VAPID_PUBLIC_KEY=...
VAPID_EMAIL=mailto:team@bazaarlens.app
ALLOWED_ORIGINS=https://bazaarlens.vercel.app,http://localhost:3000
```

---

## 18. Important Constraints

1. **No BarcodeDetector API** — use `@zxing/browser` instead. BarcodeDetector is unsupported in Safari.
2. **No native Android code** — PWA only.
3. **No user authentication** — device_id pattern only.
4. **No LangGraph / multi-agent orchestration** — single LLM call, structured JSON.
5. **No feature flags, A/B testing, or analytics** in MVP.
6. **DeepSeek V4 is primary LLM** — Gemini/Claude are credit-intensive on our OpenRouter plan.
7. **Always use `response_format: { type: "json_object" }`** — never parse free-form LLM text.
8. **Push notifications only work on iOS when PWA is installed to home screen.**

---

## 19. Things to Avoid

| Avoid | Because |
|---|---|
| User accounts / login screens | Adds friction, slows demo, not needed for MVP |
| Real-time market data scraping | Fragile, rate-limited, legally risky |
| More than one LLM call per user action | Latency kills mobile UX |
| CSS animations > 300ms duration | Feels sluggish on mobile |
| Any `console.log` left in production | Unprofessional |
| Rendering raw JSON in UI | Never acceptable |
| Native Android / React Native | Team weakness, out of scope |
| Redux or complex state management | Use React state + SWR |
| Docker for local dev | Use native Python venv + npm |
| Complex auth middleware | Not needed, device_id is enough |
| Over-engineering the admin panel | It's only for the demo |

---

## 20. Key Dependencies

### Frontend
```json
{
  "@zxing/browser": "^0.1.4",
  "next": "14.x",
  "next-pwa": "^5.6.0",
  "swr": "^2.x",
  "@supabase/supabase-js": "^2.x",
  "tailwindcss": "^3.x",
  "shadcn/ui": "latest",
  "lucide-react": "latest"
}
```

### Backend
```
fastapi>=0.110
uvicorn[standard]
httpx
pydantic-settings
supabase
pywebpush
python-dotenv
```

---

*Last updated: 2026-06-13 | Hackathon Day 1*
