use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use crate::models::patterns::{PatternLifecycleEntry, PatternLifecycleSnapshot};

/// Shared in-memory storage for pattern lifecycle entries.
#[derive(Debug)]
pub struct PatternLifecycleStateInner {
    pub entries: RwLock<Vec<PatternLifecycleEntry>>,
    pub broadcaster: broadcast::Sender<PatternLifecycleSnapshot>,
}

impl PatternLifecycleStateInner {
    pub fn new() -> Self {
        let (broadcaster, _receiver) = broadcast::channel(32);
        Self {
            entries: RwLock::new(Vec::new()),
            broadcaster,
        }
    }
}

pub type SharedPatternLifecycleState = Arc<PatternLifecycleStateInner>;
