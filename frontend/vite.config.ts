/// <reference types="vitest/config" />
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/double-top": "http://localhost:3000",
      "/double-top/stream": "http://localhost:3000",
      "/patterns": "http://localhost:3000",
      "/vwap": "http://localhost:3000",
      "/vwap/stream": "http://localhost:3000",
      "/chart": "http://localhost:3000",
      "/chart/stream": "http://localhost:3000",
      "/health": "http://localhost:3000"
    }
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/testSetup.ts"
  }
});
