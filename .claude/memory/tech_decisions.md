---
name: BazaarLens Technical Decisions
description: LLM model choices (OpenRouter), barcode scanning approach, push notification constraints, auth strategy
type: project
originSessionId: b6c1b683-14ec-4c41-bac8-a5cb287da253
---
**LLM Stack via OpenRouter credits:**
- DeepSeek V4 Pro = primary for intelligence + strategy (Recommended/free tier)
- DeepSeek V4 Flash = fast responses (Recommended/free tier)
- Minimax M3 = vision fallback for image-based product scan (Recommended/Champion)
- Gemini/Claude/GPT = Credit Intensive — avoid unless absolutely necessary

**Product Identification:**
- Barcode → Open Food Facts API first (free, no key) → UPC Item DB fallback → then LLM
- Image → Minimax M3 vision (only if no barcode detected)

**Barcode Scanning:**
- Use `@zxing/browser` NOT BarcodeDetector API (Safari doesn't support BarcodeDetector)

**Push Notifications on iOS:**
- Requires iOS 16.4+, PWA installed to home screen via Safari
- manifest.json must have `display: "standalone"`
- Permission must be requested via user gesture (button tap)
- Subscriptions can expire after 1-2 weeks
- DEMO PREP: Install PWA + grant permission BEFORE the demo

**Auth:**
- No user accounts. Use `device_id = crypto.randomUUID()` stored in localStorage
- All API calls pass X-Device-Id header

**How to apply:** Always pick Recommended OpenRouter models first. Never add login screens. Always test push on real device before demo.
