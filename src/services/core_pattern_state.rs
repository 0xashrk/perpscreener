use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::models::patterns::{PatternDetection, PatternResponse};

/// Shared in-memory storage for core pattern detections.
#[derive(Debug)]
pub struct CorePatternStateInner {
    pub detections: RwLock<Vec<PatternDetection>>,
    pub broadcaster: broadcast::Sender<PatternResponse>,
}

impl CorePatternStateInner {
    pub fn new() -> Self {
        let (broadcaster, _receiver) = broadcast::channel(32);
        Self {
            detections: RwLock::new(Vec::new()),
            broadcaster,
        }
    }
}

pub type SharedCorePatternState = Arc<CorePatternStateInner>;
