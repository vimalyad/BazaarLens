// Services are wired into AppState in main.rs and injected via Axum State.
pub mod llm;
pub mod product_lookup;
// Phase 5+ adds: push_service, event_simulator
