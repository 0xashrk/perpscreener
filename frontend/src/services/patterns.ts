import { PatternDetection, PatternSnapshot, PatternSignalType } from "../types/patterns";
import { buildApiUrl } from "./url";

type JsonValue = string | number | boolean | JsonObject | JsonValue[];

type JsonObject = {
  [key: string]: JsonValue;
};

type PatternQuery = {
  coins: string[];
  intervals: string[];
  limit: number;
  sinceMs: number;
};

const isObject = (value: JsonValue): value is JsonObject => {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
};

const isSignalType = (value: string): value is PatternSignalType => {
  return (
    value === "reversal" ||
    value === "continuation" ||
    value === "trend" ||
    value === "range" ||
    value === "key_level" ||
    value === "impulse" ||
    value === "correction"
  );
};

const isClassification = (value: string): value is PatternDetection["classification"] => {
  return value === "bullish" || value === "bearish" || value === "neutral";
};

const parseDetection = (item: JsonObject): PatternDetection | null => {
  const coin = item["coin"];
  const interval = item["interval"];
  const pattern = item["pattern"];
  const category = item["category"];
  const classification = item["classification"];
  const signalType = item["signal_type"];
  const confidence = item["confidence"];
  const detectedAtMs = item["detected_at_ms"];
  const windowStartMs = item["window_start_ms"];
  const windowEndMs = item["window_end_ms"];
  const notes = item["notes"];

  if (
    typeof coin !== "string" ||
    typeof interval !== "string" ||
    typeof pattern !== "string" ||
    typeof category !== "string" ||
    typeof classification !== "string" ||
    typeof signalType !== "string" ||
    typeof confidence !== "number" ||
    typeof detectedAtMs !== "number" ||
    typeof windowStartMs !== "number" ||
    typeof windowEndMs !== "number"
  ) {
    return null;
  }

  if (!isClassification(classification) || !isSignalType(signalType)) {
    return null;
  }

  return {
    coin,
    interval,
    pattern,
    category,
    classification,
    signalType,
    confidence,
    detectedAtMs,
    windowStartMs,
    windowEndMs,
    notes: typeof notes === "string" ? notes : ""
  };
};

const parseSnapshot = (data: JsonValue): PatternSnapshot => {
  if (!isObject(data)) {
    throw new Error("Invalid snapshot payload");
  }

  const asOfMs = data["as_of_ms"];
  const detectionsRaw = data["detections"];

  if (typeof asOfMs !== "number" || !Array.isArray(detectionsRaw)) {
    throw new Error("Missing pattern fields");
  }

  const detections: PatternDetection[] = [];
  detectionsRaw.forEach((item) => {
    if (!isObject(item)) {
      return;
    }
    const parsed = parseDetection(item);
    if (parsed) {
      detections.push(parsed);
    }
  });

  return { asOfMs, detections };
};

const buildPatternQuery = (query: PatternQuery): Record<string, string> => {
  const params: Record<string, string> = {
    limit: query.limit.toString()
  };

  if (query.coins.length > 0) {
    params["coins"] = query.coins.join(",");
  }
  if (query.intervals.length > 0) {
    params["intervals"] = query.intervals.join(",");
  }
  if (query.sinceMs > 0) {
    params["since_ms"] = query.sinceMs.toString();
  }

  return params;
};

export const fetchPatternSnapshot = async (query: PatternQuery): Promise<PatternSnapshot> => {
  const url = buildApiUrl("/patterns", buildPatternQuery(query));
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error("Failed to load patterns");
  }

  const json = (await response.json()) as JsonValue;
  return parseSnapshot(json);
};
