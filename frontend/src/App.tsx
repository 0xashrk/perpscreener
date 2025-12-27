import { useEffect, useMemo, useState } from "react";

type PatternState =
  | "WATCHING"
  | "PEAK_FOUND"
  | "TROUGH_FOUND"
  | "FORMING"
  | "CONFIRMED"
  | "INVALIDATED";

type Pattern = {
  coin: string;
  state: PatternState;
  peak1_price: number | null;
  neckline_price: number | null;
  peak2_price: number | null;
  is_warmed_up: boolean;
  summary?: string;
};

type Snapshot = {
  as_of_ms?: number;
  patterns: Pattern[];
};

const STATE_LABEL: Record<PatternState, string> = {
  WATCHING: "Watching",
  PEAK_FOUND: "Peak Found",
  TROUGH_FOUND: "Trough Found",
  FORMING: "Forming",
  CONFIRMED: "Confirmed",
  INVALIDATED: "Invalidated",
};

const STATE_TONE: Record<PatternState, string> = {
  WATCHING: "neutral",
  PEAK_FOUND: "info",
  TROUGH_FOUND: "info",
  FORMING: "warn",
  CONFIRMED: "success",
  INVALIDATED: "muted",
};

const FALLBACK_SUMMARY: Record<PatternState, (coin: string) => string> = {
  WATCHING: (coin) => `${coin}: watching for the first peak.`,
  PEAK_FOUND: (coin) => `${coin}: first peak found; waiting for pullback.`,
  TROUGH_FOUND: (coin) => `${coin}: pullback detected; watching for second peak.`,
  FORMING: (coin) => `${coin}: price approaching the first peak (early warning).`,
  CONFIRMED: (coin) => `${coin}: double top confirmed.`,
  INVALIDATED: (coin) => `${coin}: pattern invalidated; watching for new setup.`,
};

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "";

const formatPrice = (value: number | null) =>
  value === null ? "—" : `$${value.toFixed(2)}`;

export function App() {
  const [patterns, setPatterns] = useState<Pattern[]>([]);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [connection, setConnection] = useState<"live" | "reconnecting" | "offline">(
    "offline",
  );

  useEffect(() => {
    let mounted = true;
    let fallbackTimer: number | undefined;

    const fetchSnapshot = async () => {
      try {
        const response = await fetch(`${API_BASE}/double-top`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        const data = (await response.json()) as { patterns: Pattern[] };
        if (!mounted) {
          return;
        }
        setPatterns(data.patterns);
        setLastUpdated(new Date());
        setConnection("live");
      } catch {
        if (mounted) {
          setConnection("offline");
        }
      }
    };

    fetchSnapshot();

    if ("EventSource" in window) {
      const source = new EventSource(`${API_BASE}/double-top/stream`);

      source.addEventListener("snapshot", (event) => {
        try {
          const payload = JSON.parse((event as MessageEvent).data) as Snapshot;
          if (!mounted) {
            return;
          }
          setPatterns(payload.patterns ?? []);
          if (payload.as_of_ms) {
            setLastUpdated(new Date(payload.as_of_ms));
          } else {
            setLastUpdated(new Date());
          }
          setConnection("live");
        } catch {
          if (mounted) {
            setConnection("reconnecting");
          }
        }
      });

      source.onerror = () => {
        if (mounted) {
          setConnection("reconnecting");
        }
      };

      return () => {
        mounted = false;
        source.close();
        if (fallbackTimer) {
          window.clearInterval(fallbackTimer);
        }
      };
    }

    fallbackTimer = window.setInterval(fetchSnapshot, 60_000);

    return () => {
      mounted = false;
      if (fallbackTimer) {
        window.clearInterval(fallbackTimer);
      }
    };
  }, []);

  const stats = useMemo(() => {
    const warmed = patterns.filter((pattern) => pattern.is_warmed_up).length;
    const confirmed = patterns.filter((pattern) => pattern.state === "CONFIRMED").length;
    const forming = patterns.filter((pattern) => pattern.state === "FORMING").length;
    return {
      total: patterns.length,
      warmed,
      confirmed,
      forming,
    };
  }, [patterns]);

  const lastUpdatedLabel = lastUpdated
    ? lastUpdated.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    : "—";

  return (
    <div className="app">
      <header className="hero">
        <div>
          <p className="eyebrow">Perp Screener</p>
          <h1>Double Top Radar</h1>
          <p className="subtitle">
            Real-time pattern tracking from the 1m candle feed. Signals update every
            minute.
          </p>
        </div>
        <div className="hero-panel">
          <div>
            <p className="panel-label">Feed Status</p>
            <p className={`status-pill ${connection}`}>{connection}</p>
          </div>
          <div>
            <p className="panel-label">Last Update</p>
            <p className="panel-value">{lastUpdatedLabel}</p>
          </div>
          <div className="panel-divider" />
          <div>
            <p className="panel-label">Coins</p>
            <p className="panel-value">{stats.total || "—"}</p>
          </div>
          <div>
            <p className="panel-label">Warmed Up</p>
            <p className="panel-value">{stats.warmed}</p>
          </div>
          <div>
            <p className="panel-label">Forming</p>
            <p className="panel-value highlight">{stats.forming}</p>
          </div>
          <div>
            <p className="panel-label">Confirmed</p>
            <p className="panel-value highlight">{stats.confirmed}</p>
          </div>
        </div>
      </header>

      <section className="board">
        <div className="board-header">
          <h2>Live Pattern Board</h2>
          <p>Each card summarizes the current state per perp.</p>
        </div>

        <div className="grid">
          {patterns.length === 0 ? (
            <div className="empty">
              <p>No pattern data yet.</p>
              <span>Waiting for the first snapshot from the backend.</span>
            </div>
          ) : (
            patterns.map((pattern) => {
              const summary =
                pattern.summary || FALLBACK_SUMMARY[pattern.state](pattern.coin);
              return (
                <article className="card" key={pattern.coin}>
                  <div className="card-header">
                    <div>
                      <p className="coin">{pattern.coin}</p>
                      <p className="summary">{summary}</p>
                    </div>
                    <span className={`chip ${STATE_TONE[pattern.state]}`}>
                      {STATE_LABEL[pattern.state]}
                    </span>
                  </div>
                  <div className="metrics">
                    <div>
                      <p className="metric-label">Peak 1</p>
                      <p className="metric-value">
                        {formatPrice(pattern.peak1_price)}
                      </p>
                    </div>
                    <div>
                      <p className="metric-label">Neckline</p>
                      <p className="metric-value">
                        {formatPrice(pattern.neckline_price)}
                      </p>
                    </div>
                    <div>
                      <p className="metric-label">Peak 2</p>
                      <p className="metric-value">
                        {formatPrice(pattern.peak2_price)}
                      </p>
                    </div>
                  </div>
                  <div className="meta">
                    <span className={pattern.is_warmed_up ? "ready" : "warming"}>
                      {pattern.is_warmed_up ? "Warmed up" : "Warming up"}
                    </span>
                    <span className="state-code">{pattern.state}</span>
                  </div>
                </article>
              );
            })
          )}
        </div>
      </section>
    </div>
  );
}
