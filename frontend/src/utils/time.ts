export const formatAge = (asOfMs: number, nowMs: number): string => {
  if (asOfMs <= 0 || nowMs <= 0 || nowMs < asOfMs) {
    return "--";
  }

  const diff = nowMs - asOfMs;
  if (diff < 60_000) {
    return `${Math.max(1, Math.floor(diff / 1000))}s`;
  }
  if (diff < 3_600_000) {
    return `${Math.floor(diff / 60_000)}m`;
  }
  if (diff < 86_400_000) {
    return `${Math.floor(diff / 3_600_000)}h`;
  }
  return `${Math.floor(diff / 86_400_000)}d`;
};
