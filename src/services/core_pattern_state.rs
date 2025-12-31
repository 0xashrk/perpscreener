use std::sync::Arc;

use tokio::sync::RwLock;

use crate::models::patterns::PatternDetection;

/// Shared in-memory storage for core pattern detections.
#[derive(Debug)]
pub struct CorePatternStateInner {
    pub detections: RwLock<Vec<PatternDetection>>,
}

impl CorePatternStateInner {
    pub fn new() -> Self {
        Self {
            detections: RwLock::new(Vec::new()),
        }
    }
}

pub type SharedCorePatternState = Arc<CorePatternStateInner>;
