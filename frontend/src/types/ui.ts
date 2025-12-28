import { DoubleTopState } from "./doubleTop";
import { VwapPosition, VwapTimeframe } from "./vwap";

export type PatternState = {
  stateKey: DoubleTopState;
  stateLabel: string;
  lastUpdatedMs: number;
  hasData: boolean;
};

export type VwapCell = {
  position: VwapPosition;
  distancePct: number;
  hasData: boolean;
};

export type VwapTokenState = {
  lastUpdatedMs: number;
  byTimeframe: Record<VwapTimeframe, VwapCell>;
};
