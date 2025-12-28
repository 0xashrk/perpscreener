import { DoubleTopState } from "../types/doubleTop";
import { StreamStatus } from "../types/stream";
import { VwapPosition } from "../types/vwap";

export const formatDoubleTopState = (state: DoubleTopState): string => {
  switch (state) {
    case "WATCHING":
      return "Watching";
    case "PEAK_FOUND":
      return "Peak Found";
    case "TROUGH_FOUND":
      return "Trough Found";
    case "FORMING":
      return "Forming";
    case "CONFIRMED":
      return "Confirmed";
    case "INVALIDATED":
      return "Invalidated";
    default:
      return "Watching";
  }
};

export const formatStreamStatus = (status: StreamStatus): string => {
  switch (status) {
    case "open":
      return "Live";
    case "reconnecting":
      return "Reconnecting";
    case "error":
      return "Error";
    default:
      return "Connecting";
  }
};

export const formatPosition = (position: VwapPosition): string => {
  return position === "above" ? "Above" : "Below";
};

export const formatDistancePct = (distancePct: number): string => {
  const value = Math.abs(distancePct);
  const sign = distancePct >= 0 ? "+" : "-";
  return `${sign}${value.toFixed(2)}%`;
};
