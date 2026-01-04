import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Space Grotesk", "system-ui", "sans-serif"],
        display: ["Space Grotesk", "system-ui", "sans-serif"]
      },
      colors: {
        ink: {
          950: "#0B0E14",
          900: "#0E111A",
          800: "#1B1F2A",
          100: "#E6E8EE"
        }
      }
    }
  },
  plugins: []
} satisfies Config;
