use super::types::{Pivot, PivotKind, Trendline, TrendlineKind};

pub fn derive_trendlines(pivots: &[Pivot], min_points: usize) -> Vec<Trendline> {
    if min_points < 2 {
        return Vec::new();
    }

    let mut lines = Vec::new();

    if let Some(line) = line_from_last_pivots(pivots, PivotKind::Low, TrendlineKind::Support) {
        lines.push(line);
    }

    if let Some(line) = line_from_last_pivots(pivots, PivotKind::High, TrendlineKind::Resistance) {
        lines.push(line);
    }

    lines
}

fn line_from_last_pivots(
    pivots: &[Pivot],
    pivot_kind: PivotKind,
    line_kind: TrendlineKind,
) -> Option<Trendline> {
    let matching: Vec<&Pivot> = pivots.iter().filter(|p| p.kind == pivot_kind).collect();
    if matching.len() < 2 {
        return None;
    }

    let second = matching[matching.len() - 1];
    let first = matching[matching.len() - 2];

    if second.time == first.time {
        return None;
    }

    let slope = (second.price - first.price) / (second.time - first.time) as f64;
    let intercept = first.price - slope * first.time as f64;

    Some(Trendline {
        kind: line_kind,
        start_time: first.time,
        end_time: second.time,
        start_price: first.price,
        end_price: second.price,
        slope,
        intercept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_support_and_resistance_lines() {
        let pivots = vec![
            Pivot {
                index: 1,
                time: 1000,
                price: 100.0,
                kind: PivotKind::Low,
            },
            Pivot {
                index: 2,
                time: 2000,
                price: 110.0,
                kind: PivotKind::Low,
            },
            Pivot {
                index: 3,
                time: 1000,
                price: 120.0,
                kind: PivotKind::High,
            },
            Pivot {
                index: 4,
                time: 2000,
                price: 115.0,
                kind: PivotKind::High,
            },
        ];

        let lines = derive_trendlines(&pivots, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().any(|line| line.kind == TrendlineKind::Support));
        assert!(lines
            .iter()
            .any(|line| line.kind == TrendlineKind::Resistance));
    }
}
