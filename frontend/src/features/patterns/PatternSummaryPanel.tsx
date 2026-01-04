import { useRef, useState, useCallback } from "react";
import { toPng } from "html-to-image";
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { PatternSummary } from "../../types/patterns";

type PatternSummaryPanelProps = {
  summaries: PatternSummary[];
};

const formatPercent = (value: number) => `${Math.round(value * 100)}%`;

const formatInterval = (interval: string): string => {
  const map: Record<string, string> = {
    "1m": "1 Minute",
    "3m": "3 Minutes",
    "5m": "5 Minutes",
    "15m": "15 Minutes",
    "30m": "30 Minutes",
    "1h": "1 Hour",
    "2h": "2 Hours",
    "4h": "4 Hours",
    "8h": "8 Hours",
    "12h": "12 Hours",
    "1d": "1 Day",
    "3d": "3 Days",
    "1w": "1 Week",
    "1M": "1 Month",
  };
  return map[interval] ?? interval;
};

const dominantTone = (summary: PatternSummary) => {
  if (summary.bullishScore >= summary.bearishScore && summary.bullishScore >= summary.neutralScore) {
    return { label: "Bullish", tone: "text-emerald-600", value: summary.bullishScore };
  }
  if (summary.bearishScore >= summary.neutralScore) {
    return { label: "Bearish", tone: "text-rose-600", value: summary.bearishScore };
  }
  return { label: "Neutral", tone: "text-slate-500", value: summary.neutralScore };
};

const getSummaryId = (s: PatternSummary) => `${s.coin}-${s.interval}`;

type SortableTileProps = {
  summary: PatternSummary;
};

const SortableTile = ({ summary }: SortableTileProps) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: getSummaryId(summary) });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const dominant = dominantTone(summary);

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="rounded-2xl border border-slate-100 bg-white/60 p-4 cursor-grab active:cursor-grabbing"
      {...attributes}
      {...listeners}
    >
      <div className="flex items-center justify-between">
        <div className="text-sm font-semibold text-slate-900">{summary.coin}</div>
        <div className="text-[11px] font-semibold text-slate-500">
          {formatInterval(summary.interval)}
        </div>
      </div>
      <div className="mt-2 flex items-center justify-between text-xs text-slate-500">
        <span className={`font-semibold ${dominant.tone}`}>{dominant.label}</span>
        <span>{formatPercent(dominant.value)}</span>
      </div>
      <div className="mt-2 flex h-2 overflow-hidden rounded-full bg-slate-100">
        <div
          className="h-full bg-emerald-400"
          style={{ width: `${summary.bullishScore * 100}%` }}
        />
        <div
          className="h-full bg-rose-400"
          style={{ width: `${summary.bearishScore * 100}%` }}
        />
        <div
          className="h-full bg-slate-400"
          style={{ width: `${summary.neutralScore * 100}%` }}
        />
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        {summary.topSignals.length > 0 ? (
          summary.topSignals.map((signal, index) => (
            <span
              key={`${signal.pattern}-${index}`}
              className="rounded-full bg-slate-900/5 px-2.5 py-1 text-[11px] font-semibold text-slate-600"
            >
              {signal.pattern}
            </span>
          ))
        ) : (
          <span className="text-xs text-slate-400">No top signals yet.</span>
        )}
      </div>
    </div>
  );
};

export const PatternSummaryPanel = ({ summaries }: PatternSummaryPanelProps) => {
  const panelRef = useRef<HTMLDivElement>(null);
  const [isCapturing, setIsCapturing] = useState(false);

  const initialSorted = [...summaries].sort((a, b) => {
    const aTop = Math.max(a.bullishScore, a.bearishScore, a.neutralScore);
    const bTop = Math.max(b.bullishScore, b.bearishScore, b.neutralScore);
    return bTop - aTop;
  });

  const [orderedSummaries, setOrderedSummaries] = useState<PatternSummary[]>(initialSorted);

  // Keep ordered summaries in sync when new summaries arrive
  const currentIds = new Set(summaries.map(getSummaryId));
  const orderedIds = new Set(orderedSummaries.map(getSummaryId));
  
  // Add new summaries, remove stale ones
  const syncedSummaries = orderedSummaries
    .filter((s) => currentIds.has(getSummaryId(s)))
    .map((s) => summaries.find((ns) => getSummaryId(ns) === getSummaryId(s)) ?? s);
  
  // Append any new summaries not in current order
  const newSummaries = summaries.filter((s) => !orderedIds.has(getSummaryId(s)));
  const finalSummaries = [...syncedSummaries, ...newSummaries];

  if (finalSummaries.length !== orderedSummaries.length || 
      finalSummaries.some((s, i) => getSummaryId(s) !== getSummaryId(orderedSummaries[i]))) {
    setOrderedSummaries(finalSummaries);
  }

  const visibleSummaries = orderedSummaries.slice(0, 6);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      setOrderedSummaries((items) => {
        const oldIndex = items.findIndex((s) => getSummaryId(s) === active.id);
        const newIndex = items.findIndex((s) => getSummaryId(s) === over.id);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
  };

  const handleCapture = useCallback(async () => {
    if (!panelRef.current) return;
    setIsCapturing(true);
    try {
      const dataUrl = await toPng(panelRef.current, {
        backgroundColor: "#f8fafc",
        pixelRatio: 2,
      });

      // Try native share if available
      if (navigator.share && navigator.canShare) {
        const blob = await (await fetch(dataUrl)).blob();
        const file = new File([blob], "pattern-summary.png", { type: "image/png" });
        if (navigator.canShare({ files: [file] })) {
          await navigator.share({
            files: [file],
            title: "Pattern Summary",
          });
          return;
        }
      }

      // Fallback: download the image
      const link = document.createElement("a");
      link.download = "pattern-summary.png";
      link.href = dataUrl;
      link.click();
    } catch (err) {
      console.error("Failed to capture image:", err);
    } finally {
      setIsCapturing(false);
    }
  }, []);

  const visibleIds = visibleSummaries.map(getSummaryId);

  return (
    <div
      ref={panelRef}
      className="rounded-3xl border border-white/60 bg-white/70 p-5 shadow-[0_18px_40px_rgba(15,23,42,0.08)]"
    >
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.3em] text-slate-500">
            Aggregation
          </p>
          <h3 className="mt-2 text-lg font-semibold text-slate-900">Signal balance</h3>
          <p className="text-xs text-slate-500">
            Weighted summary across detected core patterns.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleCapture}
            disabled={isCapturing}
            className="rounded-full bg-slate-900/5 px-3 py-1.5 text-xs font-semibold text-slate-600 transition hover:bg-slate-900/10 active:scale-95 disabled:opacity-50"
            title="Capture and share"
          >
            {isCapturing ? (
              <span className="flex items-center gap-1.5">
                <svg className="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" opacity="0.25" />
                  <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
                </svg>
                Capturing…
              </span>
            ) : (
              <span className="flex items-center gap-1.5">
                <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                </svg>
                Share
              </span>
            )}
          </button>
          <span className="rounded-full bg-slate-900/5 px-3 py-1 text-xs font-semibold text-slate-600">
            {summaries.length} tiles
          </span>
        </div>
      </div>
      <div className="mt-4 grid gap-3">
        {visibleSummaries.length === 0 ? (
          <div className="rounded-2xl border border-dashed border-slate-200 bg-white/60 p-6 text-sm text-slate-400">
            Summaries will populate once detectors stream signals.
          </div>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext items={visibleIds} strategy={verticalListSortingStrategy}>
              {visibleSummaries.map((summary) => (
                <SortableTile key={getSummaryId(summary)} summary={summary} />
              ))}
            </SortableContext>
          </DndContext>
        )}
      </div>
    </div>
  );
};
