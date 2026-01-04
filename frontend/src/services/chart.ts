import { ChartSnapshot, Candle } from "../types/chart";
import { ParseResult } from "../types/stream";
import { buildApiUrl } from "./url";

type JsonValue = string | number | boolean | JsonObject | JsonValue[];

type JsonObject = {
  [key: string]: JsonValue;
};

const isObject = (value: JsonValue): value is JsonObject => {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
};

const parseCandle = (item: JsonObject): Candle | null => {
  const openTime = item["t"];
  const closeTime = item["T"];
  const open = item["o"];
  const high = item["h"];
  const low = item["l"];
  const close = item["c"];
  const volume = item["v"];
  const numTrades = item["n"];

  if (
    typeof openTime !== "number" ||
    typeof closeTime !== "number" ||
    typeof open !== "number" ||
    typeof high !== "number" ||
    typeof low !== "number" ||
    typeof close !== "number" ||
    typeof volume !== "number" ||
    typeof numTrades !== "number"
  ) {
    return null;
  }

  return {
    openTime,
    closeTime,
    open,
    high,
    low,
    close,
    volume,
    numTrades
  };
};

const parseSnapshot = (data: JsonValue): ChartSnapshot => {
  if (!isObject(data)) {
    throw new Error("Invalid chart snapshot payload");
  }

  const asOfMs = data["as_of_ms"];
  const coin = data["coin"];
  const interval = data["interval"];
  const candlesRaw = data["candles"];

  if (typeof asOfMs !== "number" || typeof coin !== "string" || typeof interval !== "string") {
    throw new Error("Missing chart snapshot fields");
  }

  const candles: Candle[] = [];
  if (Array.isArray(candlesRaw)) {
    candlesRaw.forEach((item) => {
      if (!isObject(item)) {
        return;
      }
      const parsed = parseCandle(item);
      if (parsed) {
        candles.push(parsed);
      }
    });
  }

  return { asOfMs, coin, interval, candles };
};

export const parseChartSnapshot = (data: string): ParseResult<ChartSnapshot> => {
  try {
    const parsed = JSON.parse(data) as JsonValue;
    return { ok: true, value: parseSnapshot(parsed) };
  } catch (error) {
    const message = error instanceof Error ? error.message : "Invalid JSON";
    return { ok: false, reason: message };
  }
};

type ChartQuery = {
  coin: string;
  interval: string;
  limit: number;
};

export const buildChartStreamUrl = (query: ChartQuery): string => {
  return buildApiUrl("/chart/stream", {
    coin: query.coin,
    interval: query.interval,
    limit: query.limit.toString()
  });
};
