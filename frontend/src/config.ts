import { VWAP_TIMEFRAMES } from "./types/vwap";

const DEFAULT_TOKENS = ["BTC", "ETH", "SOL"] as const;
const DEFAULT_VWAP_INTERVAL = "1m";

const normalizeToken = (token: string): string => token.trim().toUpperCase();

const parseTokens = (value: string, fallback: readonly string[]): string[] => {
  const tokens = value
    .split(",")
    .map(normalizeToken)
    .filter((token) => token.length > 0);

  if (tokens.length > 0) {
    return Array.from(new Set(tokens));
  }

  return [...fallback];
};

const rawBaseUrl = import.meta.env.VITE_API_BASE_URL || "";
const rawTokens = import.meta.env.VITE_TOKENS || "";

export const API_BASE_URL = rawBaseUrl.trim();
export const TOKENS = parseTokens(rawTokens, DEFAULT_TOKENS);
export const VWAP_INTERVAL = DEFAULT_VWAP_INTERVAL;
export const DEFAULT_TIMEFRAMES = [...VWAP_TIMEFRAMES];
