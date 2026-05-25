use crate::client::Candle;

pub struct VwapContext {
    pub vwap: f64,
    pub price_vs_vwap: f64,
    pub vwap_slope: f64,
    pub band_upper: f64,
    pub band_lower: f64,
}

/// Compute VWAP from today's candles (daily reset at 00:00 UTC).
/// `day_candles` must be sorted ascending and filtered to today.
pub fn compute_vwap(day_candles: &[Candle], current_price: f64) -> Option<VwapContext> {
    if day_candles.is_empty() {
        return None;
    }

    let mut cum_tp_vol = 0.0f64;
    let mut cum_vol = 0.0f64;
    let mut vwap_series = Vec::with_capacity(day_candles.len());
    let mut deviations = Vec::with_capacity(day_candles.len());

    for c in day_candles {
        let tp = (c.h + c.l + c.c) / 3.0;
        cum_tp_vol += tp * c.v;
        cum_vol += c.v;
        if cum_vol > 0.0 {
            let v = cum_tp_vol / cum_vol;
            vwap_series.push(v);
            deviations.push(tp - v);
        }
    }

    let vwap = *vwap_series.last()?;
    let price_vs_vwap = if vwap > 0.0 {
        (current_price - vwap) / vwap
    } else {
        0.0
    };

    // Slope: rate of change over last 4 candles (1 hour of 15m).
    let slope = if vwap_series.len() >= 4 {
        let tail = &vwap_series[vwap_series.len() - 4..];
        let first = tail[0];
        let last = tail[3];
        if first > 0.0 {
            (last - first) / first
        } else {
            0.0
        }
    } else if vwap_series.len() >= 2 {
        let first = vwap_series[0];
        let last = *vwap_series.last().unwrap();
        if first > 0.0 {
            (last - first) / first
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Bands: VWAP +/- 1 stddev of (typical_price - vwap).
    let n = deviations.len() as f64;
    let std = if n >= 2.0 {
        let mean = deviations.iter().sum::<f64>() / n;
        let var = deviations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0);
        var.sqrt()
    } else {
        0.0
    };

    Some(VwapContext {
        vwap,
        price_vs_vwap,
        vwap_slope: slope,
        band_upper: vwap + std,
        band_lower: vwap - std,
    })
}
