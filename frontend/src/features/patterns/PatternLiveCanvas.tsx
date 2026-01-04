import { useEffect, useMemo, useState } from "react";
import { PATTERN_INTERVALS, TOKENS } from "../../config";
import { useChartStream } from "../../hooks/useChartStream";
import { StreamStatus } from "../../types/stream";
import { formatAge } from "../../utils/time";
import { Candle } from "../../types/chart";

type CanvasSignal = {
  pattern: string;
  coin: string;
  interval: string;
  detectedAtMs: number;
  windowEndMs?: number;
};

type PatternLiveCanvasProps = {
  signals: CanvasSignal[];
  status: StreamStatus;
  lastUpdatedMs: number;
};

const statusTone = (status: StreamStatus) => {
  switch (status) {
    case "open":
      return "bg-emerald-500";
    case "reconnecting":
      return "bg-amber-500";
    case "error":
      return "bg-rose-500";
    default:
      return "bg-slate-400";
  }
};

const formatTimestamp = (value: number) => {
  if (value <= 0) {
    return "—";
  }
  return new Date(value).toLocaleTimeString();
};

const clampValue = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const buildScale = (candles: Candle[]) => {
  const highs = candles.map((c) => c.high);
  const lows = candles.map((c) => c.low);
  const max = Math.max(...highs, 0);
  const min = Math.min(...lows, max);
  const range = Math.max(1e-6, max - min);
  return { min, max, range };
};

const findClosestIndex = (timestamps: number[], target: number): number => {
  let bestIndex = 0;
  let bestDiff = Number.MAX_SAFE_INTEGER;
  timestamps.forEach((time, index) => {
    const diff = Math.abs(time - target);
    if (diff < bestDiff) {
      bestDiff = diff;
      bestIndex = index;
    }
  });
  return bestIndex;
};

const LiveCanvasChart = ({
  candles,
  signals
}: {
  candles: Candle[];
  signals: CanvasSignal[];
}) => {
  const width = 640;
  const height = 220;
  const padding = 16;
  const plotWidth = width - padding * 2;
  const plotHeight = height - padding * 2;

  if (candles.length === 0) {
    return (
      <div className="flex h-56 items-center justify-center text-sm text-slate-400">
        Awaiting chart data...
      </div>
    );
  }

  const { min, max, range } = buildScale(candles);
  const step = candles.length > 1 ? plotWidth / (candles.length - 1) : plotWidth;
  const bodyWidth = clampValue(step * 0.6, 3, 12);

  const toY = (value: number) => padding + ((max - value) / range) * plotHeight;
  const times = candles.map((candle) => candle.closeTime);

  const markers = signals.map((signal) => {
    const timestamp = signal.windowEndMs ?? signal.detectedAtMs;
    const index = findClosestIndex(times, timestamp);
    return {
      pattern: signal.pattern,
      x: padding + index * step
    };
  });

  return (
    <svg
      className="h-56 w-full"
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
    >
      <rect x="0" y="0" width={width} height={height} fill="url(#canvasGradient)" />
      <defs>
        <linearGradient id="canvasGradient" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#f8fafc" />
          <stop offset="100%" stopColor="#eef2f7" />
        </linearGradient>
      </defs>
      {candles.map((candle, index) => {
        const x = padding + index * step;
        const wickTop = toY(candle.high);
        const wickBottom = toY(candle.low);
        const bodyTop = toY(Math.max(candle.open, candle.close));
        const bodyBottom = toY(Math.min(candle.open, candle.close));
        const bodyHeight = Math.max(2, bodyBottom - bodyTop);
        const isBullish = candle.close >= candle.open;
        const color = isBullish ? "#34d399" : "#f87171";

        return (
          <g key={`${candle.closeTime}-${index}`}>
            <line
              x1={x}
              x2={x}
              y1={wickTop}
              y2={wickBottom}
              stroke={color}
              strokeWidth={2}
              opacity={0.8}
            />
            <rect
              x={x - bodyWidth / 2}
              y={bodyTop}
              width={bodyWidth}
              height={bodyHeight}
              fill={color}
              rx={2}
            />
          </g>
        );
      })}
      {markers.map((marker, index) => (
        <g key={`${marker.pattern}-${index}`}>
          <circle cx={marker.x} cy={padding} r={4} fill="#0f172a" />
          <text
            x={marker.x + 6}
            y={padding + 4}
            fontSize={10}
            fill="#0f172a"
            fontWeight={600}
          >
            {marker.pattern}
          </text>
        </g>
      ))}
    </svg>
  );
};

export const PatternLiveCanvas = ({
  signals,
  status,
  lastUpdatedMs
}: PatternLiveCanvasProps) => {
  const [selectedToken, setSelectedToken] = useState<string>(TOKENS[0] ?? "");
  const [selectedInterval, setSelectedInterval] = useState<string>(PATTERN_INTERVALS[0] ?? "1m");

  useEffect(() => {
    if (TOKENS.length > 0 && !TOKENS.includes(selectedToken)) {
      setSelectedToken(TOKENS[0]);
    }
  }, [selectedToken]);

  useEffect(() => {
    if (PATTERN_INTERVALS.length > 0 && !PATTERN_INTERVALS.includes(selectedInterval)) {
      setSelectedInterval(PATTERN_INTERVALS[0]);
    }
  }, [selectedInterval]);

  const chartStream = useChartStream(selectedToken, selectedInterval, 180);

  const filteredSignals = useMemo(() => {
    return signals
      .filter((signal) => signal.coin === selectedToken && signal.interval === selectedInterval)
      .sort((a, b) => b.detectedAtMs - a.detectedAtMs)
      .slice(0, 5);
  }, [signals, selectedInterval, selectedToken]);

  const statusLabel = status === "open" ? "Live" : status.replace(/_/g, " ");

  return (
    <div className="flex flex-col gap-4">
      <div className="rounded-3xl border border-white/60 bg-white/70 p-6 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
              Live Canvas
            </p>
            <div className="flex flex-wrap items-center gap-2 text-xs text-slate-500">
              <span className={`h-2.5 w-2.5 rounded-full ${statusTone(chartStream.status)}`} />
              <span className="uppercase tracking-[0.2em]">Chart {chartStream.status}</span>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-3 text-xs font-semibold text-slate-600">
            <label className="flex items-center gap-2">
              Token
              <select
                className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-700"
                value={selectedToken}
                onChange={(event) => setSelectedToken(event.target.value)}
              >
                {TOKENS.map((token) => (
                  <option key={token} value={token}>
                    {token}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex items-center gap-2">
              Interval
              <select
                className="rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold uppercase text-slate-700"
                value={selectedInterval}
                onChange={(event) => setSelectedInterval(event.target.value)}
              >
                {PATTERN_INTERVALS.map((interval) => (
                  <option key={interval} value={interval}>
                    {interval}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="relative h-56 overflow-hidden rounded-2xl border border-dashed border-slate-200 bg-slate-50">
            {chartStream.status === "error" ? (
              <div className="flex h-full items-center justify-center text-sm text-rose-500">
                {chartStream.error || "Chart stream error."}
              </div>
            ) : (
              <LiveCanvasChart candles={chartStream.snapshot.candles} signals={filteredSignals} />
            )}
            {filteredSignals.length === 0 ? (
              <div className="absolute bottom-3 left-3 rounded-full bg-white/80 px-3 py-1 text-xs font-semibold text-slate-500">
                No overlays yet.
              </div>
            ) : null}
          </div>
          <p className="text-sm text-slate-600">
            Overlay markers highlight the most recent detections for the selected token and interval.
          </p>
        </div>
      </div>
      <div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
        <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
          Stream Status
        </p>
        <div className="mt-3 flex flex-col gap-2 text-sm text-slate-600">
          <div className="flex items-center gap-3">
            <span className={`h-2.5 w-2.5 rounded-full ${statusTone(status)}`} />
            <span className="capitalize">Patterns: {statusLabel}</span>
          </div>
          <div className="flex items-center gap-3 text-xs text-slate-400">
            Last update: {formatTimestamp(lastUpdatedMs)} · Age {formatAge(lastUpdatedMs, Date.now())}
          </div>
        </div>
      </div>
    </div>
  );
};
