import { useEffect, useState } from "react";

const getHashRoute = () => {
  if (typeof window === "undefined") {
    return "/";
  }
  const hash = window.location.hash.replace("#", "");
  return hash.length > 0 ? hash : "/";
};

export const useHashRoute = (): string => {
  const [route, setRoute] = useState(getHashRoute);

  useEffect(() => {
    const handleChange = () => setRoute(getHashRoute());
    window.addEventListener("hashchange", handleChange);
    return () => window.removeEventListener("hashchange", handleChange);
  }, []);

  return route;
};
