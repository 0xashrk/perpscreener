/// <reference types="vitest/config" />
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/double-top": "http://localhost:30001",
      "/double-top/stream": "http://localhost:30001",
      "/patterns": "http://localhost:30001",
      "/vwap": "http://localhost:30001",
      "/vwap/stream": "http://localhost:30001",
      "/chart": "http://localhost:30001",
      "/chart/stream": "http://localhost:30001",
      "/health": "http://localhost:30001"
    }
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/testSetup.ts"
  }
});
