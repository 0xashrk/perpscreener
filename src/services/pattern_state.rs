use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::models::double_top::{CoinPatternStatus, PatternSnapshot};

/// Shared in-memory storage for pattern snapshots and SSE fanout.
#[derive(Debug)]
pub struct PatternStateInner {
    /// Latest pattern statuses.
    pub patterns: RwLock<Vec<CoinPatternStatus>>,
    /// Broadcast channel for stream updates.
    pub broadcaster: broadcast::Sender<PatternSnapshot>,
}

/// Thread-safe shared state wrapper for pattern status storage.
pub type SharedPatternState = Arc<PatternStateInner>;
