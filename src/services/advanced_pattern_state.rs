use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::models::patterns::AdvancedPatternDetection;
use crate::models::patterns::AdvancedPatternResponse;

/// Shared in-memory storage for advanced pattern detections.
#[derive(Debug)]
pub struct AdvancedPatternStateInner {
    pub detections: RwLock<Vec<AdvancedPatternDetection>>,
    pub broadcaster: broadcast::Sender<AdvancedPatternResponse>,
}

impl AdvancedPatternStateInner {
    pub fn new() -> Self {
        let (broadcaster, _receiver) = broadcast::channel(16);
        Self {
            detections: RwLock::new(Vec::new()),
            broadcaster,
        }
    }
}

pub type SharedAdvancedPatternState = Arc<AdvancedPatternStateInner>;
