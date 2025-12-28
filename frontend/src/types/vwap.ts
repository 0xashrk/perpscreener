export const VWAP_TIMEFRAMES = ["session", "1h", "4h"] as const;

export type VwapTimeframe = (typeof VWAP_TIMEFRAMES)[number];

export type VwapPosition = "above" | "below";

export type VwapEntry = {
  timeframe: VwapTimeframe;
  position: VwapPosition;
  distancePct: number;
  vwap: number;
};

export type VwapSnapshot = {
  asOfMs: number;
  coin: string;
  currentPrice: number;
  vwaps: VwapEntry[];
};
