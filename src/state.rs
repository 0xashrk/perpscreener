use std::sync::Arc;

use crate::services::hyperliquid::HyperliquidClient;
use crate::services::pattern_state::SharedPatternState;

#[derive(Clone)]
pub struct AppState {
    pub pattern_state: SharedPatternState,
    pub hyperliquid: Arc<HyperliquidClient>,
}
