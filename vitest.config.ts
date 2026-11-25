import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      enabled: true,
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/**/*.test.{ts,tsx}"],
      reportsDirectory: "coverage",
      clean: true,
      reporter: ["text", "json-summary", "lcov", "html"],
      thresholds: {
        lines: 90,
        statements: 90,
        functions: 90,
        branches: 85,
        "src/ui/ProductForm.tsx": {
          lines: 95,
          statements: 95,
          functions: 95,
          branches: 90,
          perFile: true,
        },
        "src/ui/productApi.ts": {
          lines: 95,
          statements: 95,
          functions: 95,
          branches: 90,
          perFile: true,
        },
      },
    },
  },
});
