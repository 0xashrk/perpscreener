import {
  AdvancedPatternDetection,
  AdvancedPatternSnapshot,
  PatternDetection,
  PatternSnapshot,
  PatternSignalType
} from "../types/patterns";
import { ParseResult } from "../types/stream";
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

const parseAdvancedDetection = (item: JsonObject): AdvancedPatternDetection | null => {
  const detection = parseDetection(item);
  if (!detection) {
    return null;
  }
  const method = item["method"];
  const basis = item["basis"];
  const assumptions = item["assumptions"];

  if (typeof method !== "string" || typeof basis !== "string") {
    return null;
  }

  const assumptionList: string[] = Array.isArray(assumptions)
    ? assumptions.filter((entry) => typeof entry === "string") as string[]
    : [];

  return {
    ...detection,
    method,
    basis,
    assumptions: assumptionList
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

export const parsePatternSnapshot = (data: string): ParseResult<PatternSnapshot> => {
  try {
    const parsed = JSON.parse(data) as JsonValue;
    return { ok: true, value: parseSnapshot(parsed) };
  } catch (error) {
    const message = error instanceof Error ? error.message : "Invalid JSON";
    return { ok: false, reason: message };
  }
};

export const parseAdvancedPatternSnapshot = (data: string): ParseResult<AdvancedPatternSnapshot> => {
  try {
    const parsed = JSON.parse(data) as JsonValue;
    if (!isObject(parsed)) {
      return { ok: false, reason: "Invalid snapshot payload" };
    }

    const asOfMs = parsed["as_of_ms"];
    const detectionsRaw = parsed["detections"];

    if (typeof asOfMs !== "number" || !Array.isArray(detectionsRaw)) {
      return { ok: false, reason: "Missing advanced fields" };
    }

    const detections: AdvancedPatternDetection[] = [];
    detectionsRaw.forEach((item) => {
      if (!isObject(item)) {
        return;
      }
      const parsedDetection = parseAdvancedDetection(item);
      if (parsedDetection) {
        detections.push(parsedDetection);
      }
    });

    return { ok: true, value: { asOfMs, detections } };
  } catch (error) {
    const message = error instanceof Error ? error.message : "Invalid JSON";
    return { ok: false, reason: message };
  }
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
