use axum::Json;

use crate::business_logic::patterns::lifecycle_registry::pattern_registry;
use crate::models::patterns::{PatternRegistryEntry, PatternRegistryResponse};

#[utoipa::path(
    get,
    path = "/patterns/registry",
    responses(
        (status = 200, description = "Pattern registry", body = PatternRegistryResponse)
    )
)]
/// Return the registry of known pattern state machines.
pub async fn get_pattern_registry() -> Json<PatternRegistryResponse> {
    let entries = pattern_registry()
        .into_iter()
        .map(|definition| PatternRegistryEntry {
            pattern: definition.name.to_string(),
            category: definition.category_label.to_string(),
            classification: definition.classification,
            signal_type: definition.signal_type,
            window: definition.window,
            max_age_bars: definition.max_age_bars,
        })
        .collect();

    Json(PatternRegistryResponse { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn registry_returns_expected_count() {
        let Json(payload) = get_pattern_registry().await;
        assert_eq!(payload.entries.len(), 105);
        assert!(payload
            .entries
            .iter()
            .any(|entry| entry.pattern == "Double Top"));
    }
}
