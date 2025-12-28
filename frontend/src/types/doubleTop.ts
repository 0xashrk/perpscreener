export const DOUBLE_TOP_STATES = [
  "WATCHING",
  "PEAK_FOUND",
  "TROUGH_FOUND",
  "FORMING",
  "CONFIRMED",
  "INVALIDATED"
] as const;

export type DoubleTopState = (typeof DOUBLE_TOP_STATES)[number];

export type DoubleTopPattern = {
  coin: string;
  state: DoubleTopState;
};

export type DoubleTopSnapshot = {
  asOfMs: number;
  patterns: DoubleTopPattern[];
};
