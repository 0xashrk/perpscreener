export type PatternClassification = "bullish" | "bearish" | "neutral";

export type PatternSignalType =
  | "reversal"
  | "continuation"
  | "trend"
  | "range"
  | "key_level"
  | "impulse"
  | "correction";

export type PatternDetection = {
  coin: string;
  interval: string;
  pattern: string;
  category: string;
  classification: PatternClassification;
  signalType: PatternSignalType;
  confidence: number;
  detectedAtMs: number;
  windowStartMs: number;
  windowEndMs: number;
  notes: string;
};

export type PatternLifecycleState =
  | "warming"
  | "watching"
  | "forming"
  | "confirmed"
  | "invalidated"
  | "expired";

export type PatternLifecycleEntry = {
  coin: string;
  interval: string;
  pattern: string;
  category: string;
  classification: PatternClassification;
  signalType: PatternSignalType;
  state: PatternLifecycleState;
  confidence: number;
  stateSinceMs: number;
  lastUpdatedMs: number;
  windowStartMs: number;
  windowEndMs: number;
  notes: string;
};

export type PatternLifecycleSnapshot = {
  asOfMs: number;
  entries: PatternLifecycleEntry[];
};

export type PatternRegistryEntry = {
  pattern: string;
  category: string;
  classification: PatternClassification;
  signalType: PatternSignalType;
  window: number;
  maxAgeBars: number;
};

export type PatternSnapshot = {
  asOfMs: number;
  detections: PatternDetection[];
  summaries: PatternSummary[];
};

export type PatternSummarySignal = {
  pattern: string;
  classification: PatternClassification;
  confidence: number;
};

export type PatternSummary = {
  coin: string;
  interval: string;
  bullishScore: number;
  bearishScore: number;
  neutralScore: number;
  topSignals: PatternSummarySignal[];
};

export type AdvancedPatternDetection = PatternDetection & {
  method: string;
  basis: string;
  assumptions: string[];
};

export type AdvancedPatternSnapshot = {
  asOfMs: number;
  detections: AdvancedPatternDetection[];
};
