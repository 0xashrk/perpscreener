use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::models::double_top::{CoinPatternStatus, PatternSnapshot};

#[derive(Debug)]
pub struct PatternStateInner {
    pub patterns: RwLock<Vec<CoinPatternStatus>>,
    pub broadcaster: broadcast::Sender<PatternSnapshot>,
}

pub type SharedPatternState = Arc<PatternStateInner>;
