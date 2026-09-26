import { defineConfig } from "@playwright/test";

const realAuthE2E = process.env.REAL_AUTH_E2E === "1";

const baseURL =
  process.env.E2E_BASE_URL ??
  (realAuthE2E ? "https://localhost:5173" : "http://127.0.0.1:5173");

if (realAuthE2E && !baseURL.startsWith("https://")) {
  throw new Error("REAL_AUTH_E2E requires E2E_BASE_URL to use HTTPS.");
}

export default defineConfig({
  testDir: "./tests/e2e",
  fullyParallel: true,
  forbidOnly: true,
  retries: 1,
  workers: 1,
  reporter: "list",
  use: {
    baseURL,
    ignoreHTTPSErrors: realAuthE2E,
    trace: "retain-on-failure",
  },
  webServer: realAuthE2E
    ? undefined
    : {
        command: "npm run dev -- --host 127.0.0.1",
        url: "http://127.0.0.1:5173",
        reuseExistingServer: true,
      },
});
