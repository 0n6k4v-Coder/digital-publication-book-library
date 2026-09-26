import { expect, test } from "@playwright/test";

const realAuthE2E = process.env.REAL_AUTH_E2E === "1";
const apiOrigin = process.env.E2E_API_ORIGIN ?? "https://localhost:3000";
const frontendOrigin = new URL(
  process.env.E2E_BASE_URL ?? "https://localhost:5173",
).origin;

test.describe("real HTTPS authentication session", () => {
  test.skip(
    !realAuthE2E,
    "Set REAL_AUTH_E2E=1 to run against the real HTTPS authentication stack.",
  );

  test("restores authentication after a full document reload through the browser refresh cookie", async ({
    page,
  }) => {
    const adminEmail = process.env.DEV_ADMIN_EMAIL ?? "";
    const adminPassword = process.env.DEV_ADMIN_PASSWORD ?? "";

    if (adminEmail.length === 0 || adminPassword.length === 0) {
      throw new Error(
        "DEV_ADMIN_EMAIL and DEV_ADMIN_PASSWORD must be provided for REAL_AUTH_E2E.",
      );
    }

    const refreshResponses: Array<{
      status: number;
      allowOrigin: string | undefined;
      allowCredentials: string | undefined;
    }> = [];

    const handleResponse = (response: {
      url(): string;
      request(): { method(): string };
      status(): number;
      headers(): Record<string, string>;
    }) => {
      const url = new URL(response.url());

      if (
        response.request().method() !== "POST" ||
        url.origin !== apiOrigin ||
        url.pathname !== "/auth/refresh"
      ) {
        return;
      }

      const headers = response.headers();

      refreshResponses.push({
        status: response.status(),
        allowOrigin: headers["access-control-allow-origin"],
        allowCredentials: headers["access-control-allow-credentials"],
      });
    };

    page.on("response", handleResponse);

    await page.goto("/login");

    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();

    const loginResponsePromise = page.waitForResponse((response) => {
      const url = new URL(response.url());

      return (
        response.request().method() === "POST" &&
        url.origin === apiOrigin &&
        url.pathname === "/auth/login"
      );
    });

    await page.getByLabel("Email").fill(adminEmail);
    await page.getByRole("textbox", { name: "Password" }).fill(adminPassword);

    await page.getByRole("button", { name: "Sign In" }).click();

    const loginResponse = await loginResponsePromise;

    expect(loginResponse.status()).toBe(200);

    const loginHeaders = loginResponse.headers();

    expect(loginHeaders["access-control-allow-origin"]).toBe(frontendOrigin);
    expect(loginHeaders["access-control-allow-credentials"]).toBe("true");

    await expect(page).toHaveURL(/\/admin$/);
    await expect(
      page.getByText("Digital Publication & Book Library"),
    ).toBeVisible();

    const refreshCookie = (await page.context().cookies(apiOrigin)).find(
      (cookie) => cookie.name === "__Host-refresh_token",
    );

    expect(refreshCookie).toBeDefined();
    expect(refreshCookie?.secure).toBe(true);
    expect(refreshCookie?.httpOnly).toBe(true);
    expect(refreshCookie?.sameSite).toBe("Strict");
    expect(refreshCookie?.path).toBe("/");

    const documentCookie = await page.evaluate(() => document.cookie);

    expect(documentCookie).not.toContain("__Host-refresh_token=");

    const clientStorage = await page.evaluate(() => ({
      localStorageKeys: Object.keys(localStorage),
      sessionStorageKeys: Object.keys(sessionStorage),
    }));

    expect(clientStorage.localStorageKeys).toEqual([]);
    expect(clientStorage.sessionStorageKeys).toEqual([]);

    refreshResponses.length = 0;

    await page.reload();

    await expect(page).toHaveURL(/\/admin$/);
    await expect(
      page.getByText("Digital Publication & Book Library"),
    ).toBeVisible();

    await expect
      .poll(() => refreshResponses.length, {
        timeout: 10_000,
      })
      .toBe(1);

    expect(refreshResponses[0]).toEqual({
      status: 200,
      allowOrigin: frontendOrigin,
      allowCredentials: "true",
    });

    const reloadedRefreshCookie = (
      await page.context().cookies(apiOrigin)
    ).find((cookie) => cookie.name === "__Host-refresh_token");

    expect(reloadedRefreshCookie).toBeDefined();
    expect(reloadedRefreshCookie?.secure).toBe(true);
    expect(reloadedRefreshCookie?.httpOnly).toBe(true);
    expect(reloadedRefreshCookie?.sameSite).toBe("Strict");
    expect(reloadedRefreshCookie?.path).toBe("/");

    page.off("response", handleResponse);
  });
});
