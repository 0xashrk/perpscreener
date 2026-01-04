export type Candle = {
  openTime: number;
  closeTime: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  numTrades: number;
};

export type ChartSnapshot = {
  asOfMs: number;
  coin: string;
  interval: string;
  candles: Candle[];
};
