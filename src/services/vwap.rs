use std::sync::Arc;

use anyhow::Context;
use chrono::{Datelike, Duration, Timelike, Utc};

use crate::business_logic::vwap::compute_vwap;
use crate::models::candle::Candle;
use crate::models::vwap::{VwapEntry, VwapSnapshot, VwapTimeframe};
use crate::services::hyperliquid::HyperliquidClient;

const MAX_CANDLES: u64 = 5000;

pub struct VwapService {
    client: Arc<HyperliquidClient>,
}

impl VwapService {
    pub fn new(client: Arc<HyperliquidClient>) -> Self {
        Self { client }
    }

    pub async fn fetch_snapshot(
        &self,
        coin: &str,
        interval: &str,
        interval_ms: u64,
        timeframes: &[VwapTimeframe],
        bands: bool,
    ) -> anyhow::Result<VwapSnapshot> {
        let now = chrono::Utc::now();
        let now_ms = now.timestamp_millis() as u64;
        let anchor_ms = earliest_anchor_ms(timeframes, now);

        let mut candles = self
            .client
            .fetch_candles(coin, interval, anchor_ms, now_ms)
            .await
            .context("failed to fetch candle snapshot")?;

        normalize_candles(&mut candles, coin, interval);

        let closed = filter_closed_candles(&candles, now_ms, interval_ms);
        let current_price = closed
            .last()
            .map(|candle| candle.close)
            .context("no closed candles available")?;

        let mut vwaps = Vec::with_capacity(timeframes.len());
        for timeframe in timeframes {
            let anchor_time_ms = anchor_time_ms(*timeframe, now);
            let window: Vec<Candle> = closed
                .iter()
                .filter(|candle| candle.open_time >= anchor_time_ms)
                .cloned()
                .collect();

            let result = compute_vwap(&window)
                .with_context(|| format!("no vwap data for timeframe {}", timeframe.as_str()))?;

            let distance_pct =
                ((current_price - result.vwap).abs() / result.vwap.max(f64::EPSILON)) * 100.0;
            let position = if current_price >= result.vwap {
                "above"
            } else {
                "below"
            };

            let (upper_band_1, lower_band_1, upper_band_2, lower_band_2) = if bands {
                (
                    Some(result.vwap + result.stddev),
                    Some(result.vwap - result.stddev),
                    Some(result.vwap + 2.0 * result.stddev),
                    Some(result.vwap - 2.0 * result.stddev),
                )
            } else {
                (None, None, None, None)
            };

            vwaps.push(VwapEntry {
                timeframe: timeframe.as_str().to_string(),
                anchor_time_ms,
                vwap: result.vwap,
                cumulative_volume: result.cumulative_volume,
                distance_pct,
                position: position.to_string(),
                upper_band_1,
                lower_band_1,
                upper_band_2,
                lower_band_2,
            });
        }

        Ok(VwapSnapshot {
            as_of_ms: now_ms,
            coin: coin.to_string(),
            current_price,
            vwaps,
            signals: None,
        })
    }
}

pub fn ensure_timeframes_covered(
    timeframes: &[VwapTimeframe],
    interval_ms: u64,
    now_ms: u64,
) -> Result<(), String> {
    let now_ms_i64 = i64::try_from(now_ms).map_err(|_| "invalid current time".to_string())?;
    let now = chrono::DateTime::<Utc>::from_timestamp_millis(now_ms_i64)
        .ok_or_else(|| "invalid current time".to_string())?;

    for timeframe in timeframes {
        let anchor = anchor_time_ms(*timeframe, now);
        let required = required_candles(anchor, now_ms, interval_ms);
        if required > MAX_CANDLES {
            return Err(format!(
                "timeframe {} requires {} candles with interval; max is {}",
                timeframe.as_str(),
                required,
                MAX_CANDLES
            ));
        }
    }

    Ok(())
}

pub fn required_candles(anchor_ms: u64, now_ms: u64, interval_ms: u64) -> u64 {
    if now_ms <= anchor_ms {
        return 0;
    }
    let span = now_ms - anchor_ms;
    span.div_ceil(interval_ms)
}

fn earliest_anchor_ms(timeframes: &[VwapTimeframe], now: chrono::DateTime<Utc>) -> u64 {
    timeframes
        .iter()
        .map(|timeframe| anchor_time_ms(*timeframe, now))
        .min()
        .unwrap_or_else(|| now.timestamp_millis() as u64)
}

fn anchor_time_ms(timeframe: VwapTimeframe, now: chrono::DateTime<Utc>) -> u64 {
    match timeframe {
        VwapTimeframe::Session => {
            let date = now.date_naive();
            date.and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as u64
        }
        VwapTimeframe::OneHour => {
            let date = now.date_naive();
            date.and_hms_opt(now.hour(), 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as u64
        }
        VwapTimeframe::FourHour => {
            let hour = (now.hour() / 4) * 4;
            let date = now.date_naive();
            date.and_hms_opt(hour, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as u64
        }
        VwapTimeframe::Weekly => {
            let date = now.date_naive();
            let weekday = date.weekday().num_days_from_monday() as i64;
            (date - Duration::days(weekday))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as u64
        }
        VwapTimeframe::Monthly => {
            let date = now.date_naive();
            date.with_day(1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis() as u64
        }
    }
}

fn filter_closed_candles(candles: &[Candle], now_ms: u64, interval_ms: u64) -> Vec<Candle> {
    let mut filtered: Vec<Candle> = candles
        .iter()
        .filter(|candle| candle.close_time <= now_ms.saturating_sub(interval_ms))
        .cloned()
        .collect();
    filtered.sort_by_key(|candle| candle.close_time);
    filtered
}

fn normalize_candles(candles: &mut [Candle], coin: &str, interval: &str) {
    for candle in candles {
        if candle.interval.is_none() {
            candle.interval = Some(interval.to_string());
        }
        if candle.symbol.is_none() {
            candle.symbol = Some(coin.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn required_candles_rounds_up() {
        let interval_ms = 60_000;
        let now_ms = 120_000;
        let anchor_ms = 1_000;
        let required = required_candles(anchor_ms, now_ms, interval_ms);
        assert_eq!(required, 2);
    }

    #[test]
    fn anchor_time_four_hour_aligns_to_boundary() {
        let now = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 14, 30, 0).unwrap();
        let anchor = anchor_time_ms(VwapTimeframe::FourHour, now);
        let expected = chrono::Utc
            .with_ymd_and_hms(2025, 1, 1, 12, 0, 0)
            .unwrap()
            .timestamp_millis() as u64;
        assert_eq!(anchor, expected);
    }
}
