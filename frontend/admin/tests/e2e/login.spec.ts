import { expect, test } from "@playwright/test";

const loginResponse = {
  access_token: "opaque-access-token",
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

const bootstrapResponse = {
  access_token: "bootstrapped-access-token",
  token_type: "Bearer",
  expires_in: 3600,
};

test.describe("admin login", () => {
  test("redirects unauthenticated access to /admin to /login", async ({
    page,
  }) => {
    await page.route("**/auth/refresh", async (route) => {
      expect(route.request().method()).toBe("POST");
      expect(route.request().postData()).toBeNull();
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          status: 401,
          code: "INVALID_REFRESH_TOKEN",
        }),
      });
    });

    await page.goto("/admin");

    await expect(page).toHaveURL(/\/login$/);
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
  });

  test("opens /login and completes login, admin shell, and logout", async ({
    page,
  }) => {
    await page.route("**/auth/refresh", async (route) => {
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          status: 401,
          code: "INVALID_REFRESH_TOKEN",
        }),
      });
    });

    await page.goto("/login");

    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
    await page.getByLabel("Email").fill("admin@example.com");
    await page
      .getByRole("textbox", { name: "Password" })
      .fill("example-secure-password");

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
    await expect(
      page.getByText("Digital Publication & Book Library"),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Logout" })).toBeVisible();
    await expect(page.locator("body")).not.toContainText(
      loginResponse.access_token,
    );
    await expect(page.locator("body")).not.toContainText(
      loginResponse.refresh_token,
    );

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

  test("restores authentication after a full document reload through bootstrap", async ({
    page,
  }) => {
    let refreshRequestCount = 0;

    await page.route("**/auth/refresh", async (route) => {
      refreshRequestCount += 1;
      expect(route.request().method()).toBe("POST");
      expect(route.request().postData()).toBeNull();

      if (refreshRequestCount === 1) {
        await route.fulfill({
          status: 401,
          headers: {
            "Cache-Control": "no-store",
            "Content-Type": "application/problem+json",
          },
          body: JSON.stringify({
            status: 401,
            code: "INVALID_REFRESH_TOKEN",
          }),
        });
        return;
      }

      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(bootstrapResponse),
      });
    });

    await page.goto("/login");
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();

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

    await page.getByLabel("Email").fill("admin@example.com");
    await page
      .getByRole("textbox", { name: "Password" })
      .fill("example-secure-password");
    await page.getByRole("button", { name: "Sign In" }).click();

    await expect(page).toHaveURL(/\/admin$/);

    await page.reload();

    await expect(page).toHaveURL(/\/admin$/);
    await expect(
      page.getByText("Digital Publication & Book Library"),
    ).toBeVisible();
    expect(refreshRequestCount).toBe(2);
  });

  test("shows authentication retry UI when bootstrap fails with a non-401 response", async ({
    page,
  }) => {
    await page.route("**/auth/refresh", async (route) => {
      await route.fulfill({
        status: 503,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          status: 503,
          code: "INTERNAL_SERVER_ERROR",
        }),
      });
    });

    await page.goto("/login");

    await expect(
      page.getByRole("heading", { name: "Authentication unavailable" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Retry authentication" }),
    ).toBeVisible();
  });

  test("retries bootstrap only after an explicit user action", async ({
    page,
  }) => {
    let refreshRequestCount = 0;

    await page.route("**/auth/refresh", async (route) => {
      refreshRequestCount += 1;
      expect(route.request().method()).toBe("POST");
      expect(route.request().postData()).toBeNull();

      if (refreshRequestCount === 1) {
        await route.fulfill({
          status: 503,
          headers: {
            "Cache-Control": "no-store",
            "Content-Type": "application/problem+json",
          },
          body: JSON.stringify({
            status: 503,
            code: "INTERNAL_SERVER_ERROR",
          }),
        });
        return;
      }

      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          status: 401,
          code: "INVALID_REFRESH_TOKEN",
        }),
      });
    });

    await page.goto("/login");

    await expect(
      page.getByRole("button", { name: "Retry authentication" }),
    ).toBeVisible();
    expect(refreshRequestCount).toBe(1);

    await page.getByRole("button", { name: "Retry authentication" }).click();

    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
    expect(refreshRequestCount).toBe(2);
  });

  test("rejects invalid credentials without entering the admin shell", async ({
    page,
  }) => {
    await page.route("**/auth/refresh", async (route) => {
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          status: 401,
          code: "INVALID_REFRESH_TOKEN",
        }),
      });
    });

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
    await page
      .getByRole("textbox", { name: "Password" })
      .fill("example-secure-password");
    await page.getByRole("button", { name: "Sign In" }).click();

    const alert = page.getByRole("alert");

    await expect(alert).toBeVisible();
    await expect(alert).toContainText("The email or password is incorrect.");
    await expect(page).toHaveURL(/\/login$/);
    await expect(
      page.getByText("Digital Publication & Book Library"),
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
    await expect(page.locator("body")).not.toContainText(
      "server detail must not reach the UI",
    );
  });
});
