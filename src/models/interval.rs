/// Supported candle intervals for Hyperliquid.
pub const SUPPORTED_INTERVALS: [&str; 14] = [
    "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M",
];

/// Convert a supported interval string to milliseconds.
pub fn interval_ms(interval: &str) -> Option<u64> {
    match interval {
        "1m" => Some(60_000),
        "3m" => Some(180_000),
        "5m" => Some(300_000),
        "15m" => Some(900_000),
        "30m" => Some(1_800_000),
        "1h" => Some(3_600_000),
        "2h" => Some(7_200_000),
        "4h" => Some(14_400_000),
        "8h" => Some(28_800_000),
        "12h" => Some(43_200_000),
        "1d" => Some(86_400_000),
        "3d" => Some(259_200_000),
        "1w" => Some(604_800_000),
        "1M" => Some(2_592_000_000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_ms_supports_all_intervals() {
        for interval in SUPPORTED_INTERVALS {
            assert!(
                interval_ms(interval).is_some(),
                "missing interval: {}",
                interval
            );
        }
    }

    #[test]
    fn interval_ms_rejects_unknown() {
        assert!(interval_ms("10m").is_none());
    }
}
