use std::sync::Arc;

use crate::services::hyperliquid::HyperliquidClient;
use crate::services::pattern_state::SharedPatternState;

/// Shared application state for handlers and services.
#[derive(Clone)]
pub struct AppState {
    /// In-memory pattern status store and broadcaster.
    pub pattern_state: SharedPatternState,
    /// Shared Hyperliquid API client.
    pub hyperliquid: Arc<HyperliquidClient>,
}
