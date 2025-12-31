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

export type PatternSnapshot = {
  asOfMs: number;
  detections: PatternDetection[];
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
