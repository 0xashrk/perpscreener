import { useMemo } from "react";
import { PATTERN_INTERVALS } from "../../config";
import { usePatternLifecycleStream } from "../../hooks/usePatternLifecycleStream";
import { usePatternRegistry } from "../../hooks/usePatternRegistry";
import { PatternLifecycleEntry, PatternRegistryEntry } from "../../types/patterns";
import { PatternStateMachineTable } from "./PatternStateMachineTable";

const formatClassification = (value: PatternRegistryEntry["classification"]): string => {
  return `${value.charAt(0).toUpperCase()}${value.slice(1)}`;
};

const formatCategory = (value: string): string =>
  value
    .split("_")
    .map((word) => `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
    .join(" ");

const intervalOrder = new Map(PATTERN_INTERVALS.map((interval, index) => [interval, index]));

const sortByInterval = (a: PatternLifecycleEntry, b: PatternLifecycleEntry): number => {
  const rankA = intervalOrder.get(a.interval) ?? Number.MAX_SAFE_INTEGER;
  const rankB = intervalOrder.get(b.interval) ?? Number.MAX_SAFE_INTEGER;
  if (rankA !== rankB) {
    return rankA - rankB;
  }
  return b.lastUpdatedMs - a.lastUpdatedMs;
};

const buildPatternKey = (entry: {
  pattern: string;
  category: string;
  classification: string;
}): string => `${entry.pattern}::${entry.category}::${entry.classification}`;

const buildPatternGroups = (
  registry: PatternRegistryEntry[],
  entries: PatternLifecycleEntry[],
  tokens: string[]
): Array<{ key: string; pattern: string; entriesByToken: Record<string, PatternLifecycleEntry[]> }> => {
  const tokenSet = new Set(tokens);
  const groupedEntries = new Map<
    string,
    Map<string, Map<string, PatternLifecycleEntry>>
  >();
  const patternCounts = new Map<string, number>();
  const classificationCounts = new Map<string, number>();

  registry.forEach((entry) => {
    patternCounts.set(entry.pattern, (patternCounts.get(entry.pattern) ?? 0) + 1);
    const classificationKey = `${entry.pattern}::${entry.classification}`;
    classificationCounts.set(
      classificationKey,
      (classificationCounts.get(classificationKey) ?? 0) + 1
    );
  });

  entries.forEach((entry) => {
    if (!tokenSet.has(entry.coin)) {
      return;
    }
    const key = buildPatternKey(entry);
    const tokenMap = groupedEntries.get(key) ?? new Map<string, Map<string, PatternLifecycleEntry>>();
    const intervalMap = tokenMap.get(entry.coin) ?? new Map<string, PatternLifecycleEntry>();
    const current = intervalMap.get(entry.interval);

    if (!current || entry.lastUpdatedMs > current.lastUpdatedMs) {
      intervalMap.set(entry.interval, entry);
    }

    tokenMap.set(entry.coin, intervalMap);
    groupedEntries.set(key, tokenMap);
  });

  return registry
    .map((entry) => {
      const key = buildPatternKey(entry);
      const patternCount = patternCounts.get(entry.pattern) ?? 0;
      const classificationKey = `${entry.pattern}::${entry.classification}`;
      const classificationCount = classificationCounts.get(classificationKey) ?? 0;
      let label = entry.pattern;
      if (patternCount > 1 && classificationCount > 1) {
        label = `${entry.pattern} (${formatClassification(entry.classification)} ${formatCategory(
          entry.category
        )})`;
      } else if (patternCount > 1) {
        label = `${entry.pattern} (${formatClassification(entry.classification)})`;
      }
      const tokenMap = groupedEntries.get(key);
      const entriesByToken: Record<string, PatternLifecycleEntry[]> = {};

      if (tokenMap) {
        tokenMap.forEach((intervalMap, token) => {
          const sorted = Array.from(intervalMap.values()).sort(sortByInterval);
          entriesByToken[token] = sorted;
        });
      }

      return {
        key,
        pattern: label,
        entriesByToken
      };
    })
    .sort((a, b) => a.pattern.localeCompare(b.pattern));
};

type PatternStateMachineStackProps = {
  tokens: string[];
  nowMs: number;
};

export const PatternStateMachineStack = ({
  tokens,
  nowMs
}: PatternStateMachineStackProps) => {
  const lifecycleStream = usePatternLifecycleStream();
  const registry = usePatternRegistry();

  const patternGroups = useMemo(() => {
    return buildPatternGroups(registry.entries, lifecycleStream.snapshot.entries, tokens);
  }, [lifecycleStream.snapshot.entries, registry.entries, tokens]);

  if (registry.status === "loading") {
    return (
      <section className="glass-panel rounded-3xl border border-white/70 p-6 text-sm text-slate-500 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
        Loading pattern state machines...
      </section>
    );
  }

  if (registry.status === "error") {
    return (
      <section className="glass-panel rounded-3xl border border-white/70 p-6 text-sm text-slate-500 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
        {registry.error || "Pattern registry error."}
      </section>
    );
  }

  if (patternGroups.length === 0) {
    return (
      <section className="glass-panel rounded-3xl border border-white/70 p-6 text-sm text-slate-500 shadow-[0_20px_60px_rgba(15,23,42,0.12)]">
        Pattern state machines are not available yet.
      </section>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      {patternGroups.map((group) => (
        <PatternStateMachineTable
          key={group.key}
          pattern={group.pattern}
          tokens={tokens}
          entriesByToken={group.entriesByToken}
          nowMs={nowMs}
        />
      ))}
    </div>
  );
};
