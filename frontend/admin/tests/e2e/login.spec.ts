import { expect, test } from "@playwright/test";

const loginResponse = {
  access_token: "opaque-access-token",
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

test.describe("admin login", () => {
  test("opens /login and completes login, admin shell, and logout", async ({ page }) => {
    await page.goto("/login");

    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
    await page.getByLabel("Email").fill("admin@example.com");
    await page.getByLabel("Password").fill("example-secure-password");

    await page.route("**/auth/login", async (route) => {
      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(loginResponse),
      });
    });

    await page.getByRole("button", { name: "Sign In" }).click();

    await expect(page).toHaveURL(/\/admin$/);
    await expect(page.getByText("Digital Publication & Book Library")).toBeVisible();
    await expect(page.getByRole("button", { name: "Logout" })).toBeVisible();
    await expect(page.locator("body")).not.toContainText(loginResponse.access_token);
    await expect(page.locator("body")).not.toContainText(loginResponse.refresh_token);

    await page.route("**/auth/logout", async (route) => {
      expect(route.request().method()).toBe("POST");
      expect(route.request().headers().authorization).toBe(
        `Bearer ${loginResponse.access_token}`,
      );
      expect(route.request().url()).not.toContain(loginResponse.access_token);
      await route.fulfill({
        status: 204,
        headers: { "Cache-Control": "no-store" },
      });
    });

    await page.getByRole("button", { name: "Logout" }).click();

    await expect(page).toHaveURL(/\/login$/);
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
  });

  test("rejects invalid credentials without entering the admin shell", async ({ page }) => {
    await page.goto("/login");

    await page.route("**/auth/login", async (route) => {
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/invalid-credentials",
          title: "Authentication failed",
          status: 401,
          detail: "server detail must not reach the UI",
          code: "INVALID_CREDENTIALS",
        }),
      });
    });

    await page.getByLabel("Email").fill("admin@example.com");
    await page.getByLabel("Password").fill("example-secure-password");
    await page.getByRole("button", { name: "Sign In" }).click();

    await expect(page.getByRole("alert")).toHaveText("The email or password is incorrect.");
    await expect(page).toHaveURL(/\/login$/);
    await expect(page.getByText("Digital Publication & Book Library")).not.toBeVisible();
    await expect(page.locator("body")).not.toContainText("server detail must not reach the UI");
  });
});