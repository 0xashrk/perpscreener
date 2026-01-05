import {
	type CandlestickData,
	ColorType,
	CrosshairMode,
	createChart,
	type IChartApi,
	type ISeriesApi,
	type SeriesMarker,
	type UTCTimestamp,
} from "lightweight-charts";
import { useEffect, useMemo, useRef, useState } from "react";
import { PATTERN_INTERVALS, TOKENS } from "../../config";
import { useChartStream } from "../../hooks/useChartStream";
import type { Candle } from "../../types/chart";
import type { StreamStatus } from "../../types/stream";
import { formatAge } from "../../utils/time";

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

const MAX_CHART_BARS = 180;
const MIN_CHART_BARS = 30;
const TARGET_RANGE_DAYS = 365;

const intervalToMinutes = (interval: string): number | null => {
	const match = interval.match(/^(\d+)([mhdwM])$/);
	if (!match) {
		return null;
	}

	const amount = Number(match[1]);
	if (!Number.isFinite(amount) || amount <= 0) {
		return null;
	}

	switch (match[2]) {
		case "m":
			return amount;
		case "h":
			return amount * 60;
		case "d":
			return amount * 1440;
		case "w":
			return amount * 10080;
		case "M":
			return amount * 43200;
		default:
			return null;
	}
};

const chartLimitForInterval = (interval: string): number => {
	const minutes = intervalToMinutes(interval);
	if (!minutes) {
		return MAX_CHART_BARS;
	}

	const barsForRange = Math.floor((TARGET_RANGE_DAYS * 1440) / minutes);
	return Math.min(MAX_CHART_BARS, Math.max(MIN_CHART_BARS, barsForRange));
};

const toTimestamp = (valueMs: number): UTCTimestamp => {
	return Math.max(0, Math.floor(valueMs / 1000)) as UTCTimestamp;
};

const sortCandles = (candles: Candle[]): Candle[] => {
	return [...candles].sort((a, b) => a.closeTime - b.closeTime);
};

const buildSeriesData = (
	candles: Candle[],
): CandlestickData<UTCTimestamp>[] => {
	return candles.map((candle) => ({
		time: toTimestamp(candle.closeTime),
		open: candle.open,
		high: candle.high,
		low: candle.low,
		close: candle.close,
	}));
};

const findClosestCandle = (
	candles: Candle[],
	targetMs: number,
): Candle | null => {
	if (candles.length === 0) {
		return null;
	}

	let closest = candles[0];
	let bestDiff = Math.abs(closest.closeTime - targetMs);

	candles.forEach((candle) => {
		const diff = Math.abs(candle.closeTime - targetMs);
		if (diff < bestDiff) {
			bestDiff = diff;
			closest = candle;
		}
	});

	return closest;
};

const buildMarkers = (
	signals: CanvasSignal[],
	candles: Candle[],
): SeriesMarker<UTCTimestamp>[] => {
	if (signals.length === 0 || candles.length === 0) {
		return [];
	}

	return signals
		.map((signal): SeriesMarker<UTCTimestamp> | null => {
			const target = signal.windowEndMs ?? signal.detectedAtMs;
			const closest = findClosestCandle(candles, target);
			if (!closest) {
				return null;
			}

			return {
				time: toTimestamp(closest.closeTime),
				position: "aboveBar" as const,
				color: "#0f172a",
				shape: "circle" as const,
				text: signal.pattern,
			};
		})
		.filter((marker): marker is SeriesMarker<UTCTimestamp> => Boolean(marker));
};

const LiveCanvasChart = ({
	candles,
	signals,
}: {
	candles: Candle[];
	signals: CanvasSignal[];
}) => {
	const containerRef = useRef<HTMLDivElement | null>(null);
	const chartRef = useRef<IChartApi | null>(null);
	const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);

	const sortedCandles = useMemo(() => sortCandles(candles), [candles]);
	const seriesData = useMemo(
		() => buildSeriesData(sortedCandles),
		[sortedCandles],
	);
	const markers = useMemo(
		() => buildMarkers(signals, sortedCandles),
		[signals, sortedCandles],
	);

	useEffect(() => {
		const container = containerRef.current;
		if (!container) {
			return undefined;
		}

		const width = Math.max(320, container.clientWidth);
		const height = Math.max(180, container.clientHeight);

		const chart = createChart(container, {
			width,
			height,
			layout: {
				background: { type: ColorType.Solid, color: "#f8fafc" },
				textColor: "#475569",
				fontFamily: '"Space Grotesk", ui-sans-serif, system-ui',
			},
			grid: {
				vertLines: { color: "#e2e8f0" },
				horzLines: { color: "#e2e8f0" },
			},
			crosshair: { mode: CrosshairMode.Normal },
			rightPriceScale: { borderColor: "#e2e8f0" },
			timeScale: { borderColor: "#e2e8f0", timeVisible: true },
		});

		const series = chart.addCandlestickSeries({
			upColor: "#34d399",
			downColor: "#f87171",
			borderVisible: false,
			wickUpColor: "#34d399",
			wickDownColor: "#f87171",
		});

		chartRef.current = chart;
		seriesRef.current = series;

		const handleResize = () => {
			if (!containerRef.current || !chartRef.current) {
				return;
			}
			const nextWidth = Math.max(320, containerRef.current.clientWidth);
			const nextHeight = Math.max(180, containerRef.current.clientHeight);
			chartRef.current.applyOptions({ width: nextWidth, height: nextHeight });
		};

		let resizeObserver: ResizeObserver | null = null;
		if (typeof ResizeObserver !== "undefined") {
			resizeObserver = new ResizeObserver(handleResize);
			resizeObserver.observe(container);
		} else {
			window.addEventListener("resize", handleResize);
		}

		return () => {
			if (resizeObserver) {
				resizeObserver.disconnect();
			} else {
				window.removeEventListener("resize", handleResize);
			}
			chart.remove();
		};
	}, []);

	useEffect(() => {
		if (!seriesRef.current) {
			return;
		}

		seriesRef.current.setData(seriesData);
		chartRef.current?.timeScale().fitContent();
	}, [seriesData]);

	useEffect(() => {
		if (!seriesRef.current) {
			return;
		}

		seriesRef.current.setMarkers(markers);
	}, [markers]);

	return (
		<div className="relative h-56 w-full">
			<div ref={containerRef} className="h-full w-full" />
			{seriesData.length === 0 ? (
				<div className="absolute inset-0 flex items-center justify-center text-sm text-slate-400">
					Awaiting chart data...
				</div>
			) : null}
		</div>
	);
};

export const PatternLiveCanvas = ({
	signals,
	status,
	lastUpdatedMs,
}: PatternLiveCanvasProps) => {
	const [selectedToken, setSelectedToken] = useState<string>(TOKENS[0] ?? "");
	const [selectedInterval, setSelectedInterval] = useState<string>(
		PATTERN_INTERVALS[0] ?? "1m",
	);

	useEffect(() => {
		if (TOKENS.length > 0 && !TOKENS.includes(selectedToken)) {
			setSelectedToken(TOKENS[0]);
		}
	}, [selectedToken]);

	useEffect(() => {
		if (
			PATTERN_INTERVALS.length > 0 &&
			!PATTERN_INTERVALS.includes(selectedInterval)
		) {
			setSelectedInterval(PATTERN_INTERVALS[0]);
		}
	}, [selectedInterval]);

	const chartLimit = useMemo(
		() => chartLimitForInterval(selectedInterval),
		[selectedInterval],
	);
	const chartStream = useChartStream(
		selectedToken,
		selectedInterval,
		chartLimit,
	);

	const filteredSignals = useMemo(() => {
		return signals
			.filter(
				(signal) =>
					signal.coin === selectedToken && signal.interval === selectedInterval,
			)
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
							<span
								className={`h-2.5 w-2.5 rounded-full ${statusTone(chartStream.status)}`}
							/>
							<span className="uppercase tracking-[0.2em]">
								Chart {chartStream.status}
							</span>
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
							<LiveCanvasChart
								candles={chartStream.snapshot.candles}
								signals={filteredSignals}
							/>
						)}
						{chartStream.status !== "error" && filteredSignals.length === 0 ? (
							<div className="absolute bottom-3 left-3 rounded-full bg-white/80 px-3 py-1 text-xs font-semibold text-slate-500">
								No overlays yet.
							</div>
						) : null}
					</div>
					<p className="text-sm text-slate-600">
						Overlay markers highlight the most recent detections for the
						selected token and interval.
					</p>
					{filteredSignals.length > 0 ? (
						<div className="flex flex-wrap items-center gap-2 text-xs font-semibold text-slate-600">
							<span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-slate-400">
								Recent overlays
							</span>
							{filteredSignals.map((signal) => (
								<span
									key={`${signal.pattern}-${signal.detectedAtMs}`}
									className="rounded-full bg-slate-100 px-3 py-1"
								>
									{signal.pattern} ·{" "}
									{formatTimestamp(signal.windowEndMs ?? signal.detectedAtMs)}
								</span>
							))}
						</div>
					) : null}
				</div>
			</div>
			<div className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]">
				<p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
					Stream Status
				</p>
				<div className="mt-3 flex flex-col gap-2 text-sm text-slate-600">
					<div className="flex items-center gap-3">
						<span
							className={`h-2.5 w-2.5 rounded-full ${statusTone(status)}`}
						/>
						<span className="capitalize">Patterns: {statusLabel}</span>
					</div>
					<div className="flex items-center gap-3 text-xs text-slate-400">
						Last update: {formatTimestamp(lastUpdatedMs)} · Age{" "}
						{formatAge(lastUpdatedMs, Date.now())}
					</div>
				</div>
			</div>
		</div>
	);
};
