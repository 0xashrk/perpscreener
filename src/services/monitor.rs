use crate::business_logic::config::DoubleTopConfig;
use crate::business_logic::double_top::{Alert, DoubleTopDetector, PatternState};
use crate::models::double_top::{CoinPatternStatus, PatternSnapshot};
use crate::services::hyperliquid::HyperliquidClient;
use crate::services::pattern_state::SharedPatternState;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

const INTERVAL_MS: u64 = 60_000; // 1 minute

/// Monitoring service that runs double top detection for multiple coins
pub struct MonitorService {
    client: HyperliquidClient,
    detectors: HashMap<String, DoubleTopDetector>,
    config: DoubleTopConfig,
    last_candle_time: HashMap<String, u64>,
    shared_state: SharedPatternState,
}

impl MonitorService {
    pub fn new(
        coins: Vec<String>,
        config: DoubleTopConfig,
        shared_state: SharedPatternState,
    ) -> Self {
        let mut detectors = HashMap::new();
        for coin in coins {
            detectors.insert(coin.clone(), DoubleTopDetector::new(coin, config.clone()));
        }

        Self {
            client: HyperliquidClient::new(),
            detectors,
            config,
            last_candle_time: HashMap::new(),
            shared_state,
        }
    }

    /// Initialize detectors with historical data
    pub async fn warmup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let coins: Vec<String> = self.detectors.keys().cloned().collect();

        for coin in coins {
            tracing::info!("Warming up detector for {}", coin);

            match self
                .client
                .fetch_warmup_candles(&coin, self.config.warmup_candles)
                .await
            {
                Ok(candles) => {
                    let now = chrono::Utc::now().timestamp_millis() as u64;

                    let mut alerts = Vec::new();
                    let mut processed = 0;
                    let mut last_close_time = None;
                    let mut final_state = None;

                    if let Some(detector) = self.detectors.get_mut(&coin) {
                        for candle in &candles {
                            // Only process closed candles
                            if candle.close_time <= now - INTERVAL_MS {
                                if let Some(alert) = detector.process_candle(candle) {
                                    alerts.push(alert);
                                }
                                processed += 1;
                                last_close_time = Some(candle.close_time);
                            }
                        }
                        final_state = Some(detector.state());
                    }

                    // Handle alerts outside the borrow
                    for alert in alerts {
                        Self::log_alert(&alert);
                    }

                    if let Some(close_time) = last_close_time {
                        self.last_candle_time.insert(coin.clone(), close_time);
                    }

                    tracing::info!(
                        "Warmed up {} with {} candles (state: {:?})",
                        coin,
                        processed,
                        final_state
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to warmup {}: {}", coin, e);
                }
            }
        }

        // Update shared state after warmup
        self.update_shared_state().await;

        Ok(())
    }

    /// Start the monitoring loop
    pub async fn run(&mut self) {
        let mut ticker = interval(Duration::from_secs(60));

        loop {
            ticker.tick().await;

            let coins: Vec<String> = self.detectors.keys().cloned().collect();

            for coin in coins {
                if let Err(e) = self.process_coin(&coin).await {
                    tracing::error!("Error processing {}: {}", coin, e);
                }
            }

            // Update shared state after each cycle
            self.update_shared_state().await;
        }
    }

    async fn process_coin(
        &mut self,
        coin: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp_millis() as u64;

        // Fetch recent candles
        let start_time = self
            .last_candle_time
            .get(coin)
            .copied()
            .unwrap_or(now - INTERVAL_MS * 5);

        let candles = self
            .client
            .fetch_candles(coin, "1m", start_time, now)
            .await?;

        let mut alerts = Vec::new();
        let last_time = self.last_candle_time.get(coin).copied().unwrap_or(0);

        if let Some(detector) = self.detectors.get_mut(coin) {
            for candle in &candles {
                // Only process closed candles we haven't seen
                if candle.close_time <= now - INTERVAL_MS && candle.close_time > last_time {
                    if let Some(alert) = detector.process_candle(candle) {
                        alerts.push(alert);
                    }
                    self.last_candle_time
                        .insert(coin.to_string(), candle.close_time);
                }
            }
        }

        // Handle alerts outside the borrow
        for alert in alerts {
            Self::log_alert(&alert);
        }

        Ok(())
    }

    async fn update_shared_state(&self) {
        let mut statuses = Vec::new();

        for (coin, detector) in &self.detectors {
            statuses.push(CoinPatternStatus {
                coin: coin.clone(),
                state: detector.state().into(),
                peak1_price: detector.peak1_price(),
                neckline_price: detector.neckline_price(),
                peak2_price: detector.peak2_price(),
                is_warmed_up: detector.is_warmed_up(),
                summary: build_summary(
                    coin,
                    detector.state(),
                    detector.peak1_price(),
                    detector.neckline_price(),
                    detector.is_warmed_up(),
                ),
            });
        }

        // Sort by coin name for consistent ordering
        statuses.sort_by(|a, b| a.coin.cmp(&b.coin));

        let snapshot = PatternSnapshot {
            as_of_ms: chrono::Utc::now().timestamp_millis() as u64,
            patterns: statuses.clone(),
        };

        let mut state = self.shared_state.patterns.write().await;
        *state = statuses;
        let _ = self.shared_state.broadcaster.send(snapshot);
    }

    fn log_alert(alert: &Alert) {
        match alert {
            Alert::EarlyWarning {
                coin,
                peak_price,
                current_price,
            } => {
                tracing::warn!(
                    "🔔 EARLY WARNING: Potential double top forming on {} - price ${:.2} approaching previous high of ${:.2}",
                    coin,
                    current_price,
                    peak_price
                );
            }
            Alert::Confirmation {
                coin,
                neckline_price,
                break_price,
            } => {
                tracing::warn!(
                    "🚨 CONFIRMED: Double top on {} - broke neckline at ${:.2} (break: ${:.2})",
                    coin,
                    neckline_price,
                    break_price
                );
            }
        }
    }
}

fn build_summary(
    coin: &str,
    state: PatternState,
    peak1_price: Option<f64>,
    neckline_price: Option<f64>,
    is_warmed_up: bool,
) -> String {
    if !is_warmed_up {
        return format!("{coin}: warming up, collecting candles before detection.");
    }

    match state {
        PatternState::Watching => format!("{coin}: watching for the first peak."),
        PatternState::PeakFound => match peak1_price {
            Some(price) => format!(
                "{coin}: first peak found at ${}; waiting for pullback.",
                format_price(price)
            ),
            None => format!("{coin}: first peak found; waiting for pullback."),
        },
        PatternState::TroughFound => match (peak1_price, neckline_price) {
            (Some(peak), Some(trough)) => format!(
                "{coin}: trough at ${} after peak at ${}; watching for second peak.",
                format_price(trough),
                format_price(peak)
            ),
            (Some(peak), None) => format!(
                "{coin}: pullback detected after peak at ${}; watching for second peak.",
                format_price(peak)
            ),
            _ => format!("{coin}: pullback detected; watching for second peak."),
        },
        PatternState::Forming => match peak1_price {
            Some(price) => format!(
                "{coin}: price is approaching the first peak near ${} (early warning).",
                format_price(price)
            ),
            None => format!("{coin}: price is approaching the first peak (early warning)."),
        },
        PatternState::Confirmed => match neckline_price {
            Some(trough) => format!(
                "{coin}: double top confirmed; broke neckline near ${}.",
                format_price(trough)
            ),
            None => format!("{coin}: double top confirmed."),
        },
        PatternState::Invalidated => match peak1_price {
            Some(price) => format!(
                "{coin}: pattern invalidated after peak at ${}; watching for new setup.",
                format_price(price)
            ),
            None => format!("{coin}: pattern invalidated; watching for new setup."),
        },
    }
}

fn format_price(price: f64) -> String {
    format!("{:.2}", price)
}
