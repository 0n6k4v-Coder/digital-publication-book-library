import { expect, test, type Page } from "@playwright/test";

const loginResponse = {
  access_token: "opaque-access-token",
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

const accountsResponse = {
  items: [
    {
      id: "01900000-0000-7000-8000-000000000001",
      email: "admin@example.com",
      status: "active",
      created_at: "2026-09-23T10:00:00Z",
      updated_at: "2026-09-23T10:00:00Z",
      deleted_at: null,
    },
  ],
  page: 1,
  page_size: 20,
  total: 1,
};

async function authenticate(page: Page): Promise<void> {
  await page.goto("/login");

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
}

test.describe("admin accounts", () => {
  test("redirects unauthenticated access to /admin/accounts", async ({
    page,
  }) => {
    await page.goto("/admin/accounts");

    await expect(page).toHaveURL(/\/login$/);
    await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
  });

  test("renders the account list inside the persistent admin shell", async ({
    page,
  }) => {
    await authenticate(page);

    await page.route("**/admin/accounts**", async (route) => {
      expect(route.request().method()).toBe("GET");
      expect(route.request().headers().authorization).toBe(
        `Bearer ${loginResponse.access_token}`,
      );
      expect(route.request().url()).not.toContain(loginResponse.access_token);

      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(accountsResponse),
      });
    });

    await page.getByRole("link", { name: "Accounts" }).click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);
    await expect(
      page.getByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeVisible();

    await expect(page.getByRole("button", { name: "Logout" })).toBeVisible();

    const accountsLink = page.getByRole("link", { name: "Accounts" });

    await expect(accountsLink).toHaveAttribute("aria-current", "page");

    await expect(
      page.getByRole("cell", {
        name: "admin@example.com",
      }),
    ).toBeVisible();

    await expect(page.locator("body")).not.toContainText(
      loginResponse.access_token,
    );
    await expect(page.locator("body")).not.toContainText(
      loginResponse.refresh_token,
    );

    await page.goBack();

    await expect(page).toHaveURL(/\/admin$/);
    await expect(
      page.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).not.toBeVisible();
  });

  test.describe("mobile navigation", () => {
    test.use({
      viewport: {
        width: 390,
        height: 844,
      },
    });

    test("can be dismissed without leaving /admin/accounts", async ({
      page,
    }) => {
      await authenticate(page);

      await page.route("**/admin/accounts**", async (route) => {
        await route.fulfill({
          status: 200,
          headers: {
            "Cache-Control": "no-store",
            "Content-Type": "application/json",
          },
          body: JSON.stringify(accountsResponse),
        });
      });

      const menuButton = page.getByRole("button", {
        name: /^(Open|Close) admin navigation$/,
      });

      await menuButton.click();

      await expect(menuButton).toHaveAttribute("aria-expanded", "true");

      await page.getByRole("link", { name: "Accounts" }).click();

      await expect(page).toHaveURL(/\/admin\/accounts$/);
      await expect(menuButton).toHaveAttribute("aria-expanded", "false");

      await menuButton.click();

      await expect(menuButton).toHaveAttribute("aria-expanded", "true");

      await page
        .getByRole("button", { name: "Dismiss admin navigation" })
        .click();

      await expect(page).toHaveURL(/\/admin\/accounts$/);
      await expect(menuButton).toHaveAttribute("aria-expanded", "false");
    });
  });
});
