import { API_BASE_URL } from "../config";

export const buildApiUrl = (path: string, query: Record<string, string>): string => {
  const base = API_BASE_URL.length > 0 ? API_BASE_URL : window.location.origin;
  const url = new URL(path, base);

  Object.entries(query).forEach(([key, value]) => {
    url.searchParams.set(key, value);
  });

  return url.toString();
};
