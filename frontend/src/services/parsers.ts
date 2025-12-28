import { DoubleTopSnapshot, DOUBLE_TOP_STATES, DoubleTopState } from "../types/doubleTop";
import { ParseResult } from "../types/stream";
import {
  VwapSnapshot,
  VwapTimeframe,
  VwapPosition,
  VWAP_TIMEFRAMES
} from "../types/vwap";

type JsonValue = string | number | boolean | JsonObject | JsonValue[];

type JsonObject = {
  [key: string]: JsonValue;
};

const isObject = (value: JsonValue): value is JsonObject => {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
};

const isDoubleTopState = (value: string): value is DoubleTopState => {
  return DOUBLE_TOP_STATES.includes(value as DoubleTopState);
};

const isVwapTimeframe = (value: string): value is VwapTimeframe => {
  return VWAP_TIMEFRAMES.includes(value as VwapTimeframe);
};

const isPosition = (value: string): value is VwapPosition => {
  return value === "above" || value === "below";
};

export const parseDoubleTopSnapshot = (data: string): ParseResult<DoubleTopSnapshot> => {
  try {
    const parsed = JSON.parse(data) as JsonObject;
    const asOfMs = parsed["as_of_ms"];
    const patternsRaw = parsed["patterns"];

    if (typeof asOfMs !== "number" || !Array.isArray(patternsRaw)) {
      return { ok: false, reason: "Missing as_of_ms or patterns" };
    }

    const patterns: { coin: string; state: DoubleTopState }[] = [];
    patternsRaw.forEach((item) => {
      if (!isObject(item)) {
        return;
      }
      const coin = item["coin"];
      const state = item["state"];
      if (typeof coin !== "string" || typeof state !== "string") {
        return;
      }
      if (!isDoubleTopState(state)) {
        return;
      }
      patterns.push({ coin, state });
    });

    return { ok: true, value: { asOfMs, patterns } };
  } catch {
    return { ok: false, reason: "Invalid JSON" };
  }
};

export const parseVwapSnapshot = (data: string): ParseResult<VwapSnapshot> => {
  try {
    const parsed = JSON.parse(data) as JsonObject;
    const asOfMs = parsed["as_of_ms"];
    const coin = parsed["coin"];
    const currentPrice = parsed["current_price"];
    const vwapsRaw = parsed["vwaps"];

    if (
      typeof asOfMs !== "number" ||
      typeof coin !== "string" ||
      typeof currentPrice !== "number" ||
      !Array.isArray(vwapsRaw)
    ) {
      return { ok: false, reason: "Missing VWAP fields" };
    }

    const vwaps: {
      timeframe: VwapTimeframe;
      position: VwapPosition;
      distancePct: number;
      vwap: number;
    }[] = [];
    vwapsRaw.forEach((item) => {
      if (!isObject(item)) {
        return;
      }
      const timeframe = item["timeframe"];
      const position = item["position"];
      const distancePct = item["distance_pct"];
      const vwap = item["vwap"];

      if (
        typeof timeframe !== "string" ||
        typeof position !== "string" ||
        typeof distancePct !== "number" ||
        typeof vwap !== "number"
      ) {
        return;
      }
      if (!isVwapTimeframe(timeframe) || !isPosition(position)) {
        return;
      }

      vwaps.push({ timeframe, position, distancePct, vwap });
    });

    return { ok: true, value: { asOfMs, coin, currentPrice, vwaps } };
  } catch {
    return { ok: false, reason: "Invalid JSON" };
  }
};
