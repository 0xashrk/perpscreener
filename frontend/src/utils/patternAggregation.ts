import {
  PatternDetection,
  PatternSummary,
  PatternSummarySignal,
  PatternSignalType
} from "../types/patterns";

export type PatternWeightConfig = {
  timeframe: Record<string, number>;
  signalType: Record<PatternSignalType, number>;
};

export const DEFAULT_TIMEFRAME_WEIGHTS: Record<string, number> = {
  "1m": 0.5,
  "3m": 0.6,
  "5m": 0.7,
  "15m": 0.8,
  "30m": 0.9,
  "1h": 1.0,
  "2h": 1.1,
  "4h": 1.2,
  "8h": 1.3,
  "12h": 1.4,
  "1d": 1.6,
  "3d": 1.8,
  "1w": 2.0,
  "1M": 2.2
};

export const DEFAULT_SIGNAL_WEIGHTS: Record<PatternSignalType, number> = {
  reversal: 0.9,
  continuation: 1.1,
  trend: 1.2,
  range: 0.7,
  key_level: 1.0,
  impulse: 1.3,
  correction: 0.8
};

export const createDefaultWeightConfig = (): PatternWeightConfig => ({
  timeframe: { ...DEFAULT_TIMEFRAME_WEIGHTS },
  signalType: { ...DEFAULT_SIGNAL_WEIGHTS }
});

type SummaryBucket = {
  coin: string;
  interval: string;
  bullish: number;
  bearish: number;
  neutral: number;
  scored: Array<{ score: number; detection: PatternDetection }>;
};

const computeScore = (detection: PatternDetection, weights: PatternWeightConfig): number => {
  const timeframeWeight = weights.timeframe[detection.interval] ?? 1;
  const signalWeight = weights.signalType[detection.signalType] ?? 1;
  return detection.confidence * timeframeWeight * signalWeight;
};

export const summarizeDetections = (
  detections: PatternDetection[],
  weights: PatternWeightConfig
): PatternSummary[] => {
  const buckets = new Map<string, SummaryBucket>();

  detections.forEach((detection) => {
    const key = `${detection.coin}-${detection.interval}`;
    const bucket = buckets.get(key) ?? {
      coin: detection.coin,
      interval: detection.interval,
      bullish: 0,
      bearish: 0,
      neutral: 0,
      scored: []
    };

    const score = computeScore(detection, weights);
    if (detection.classification === "bullish") {
      bucket.bullish += score;
    } else if (detection.classification === "bearish") {
      bucket.bearish += score;
    } else {
      bucket.neutral += score;
    }

    bucket.scored.push({ score, detection });
    buckets.set(key, bucket);
  });

  const summaries: PatternSummary[] = [];

  buckets.forEach((bucket) => {
    const total = bucket.bullish + bucket.bearish + bucket.neutral;
    const bullishScore = total > 0 ? bucket.bullish / total : 0;
    const bearishScore = total > 0 ? bucket.bearish / total : 0;
    const neutralScore = total > 0 ? bucket.neutral / total : 0;

    const topSignals: PatternSummarySignal[] = [...bucket.scored]
      .sort((a, b) => b.score - a.score)
      .slice(0, 3)
      .map(({ detection }) => ({
        pattern: detection.pattern,
        classification: detection.classification,
        confidence: detection.confidence
      }));

    summaries.push({
      coin: bucket.coin,
      interval: bucket.interval,
      bullishScore,
      bearishScore,
      neutralScore,
      topSignals
    });
  });

  summaries.sort((a, b) => {
    if (a.coin !== b.coin) {
      return a.coin.localeCompare(b.coin);
    }
    return a.interval.localeCompare(b.interval);
  });

  return summaries;
};
