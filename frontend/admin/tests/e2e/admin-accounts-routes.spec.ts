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
      display_name: "Library Administrator",
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

async function stubAccountsApi(page: Page): Promise<void> {
  await page.route(/\/admin\/accounts(?:\?.*)?$/, async (route) => {
    if (route.request().resourceType() !== "fetch") {
      await route.continue();
      return;
    }

    await route.fulfill({
      status: 200,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
      },
      body: JSON.stringify(accountsResponse),
    });
  });
}

test.describe("admin accounts routes", () => {
  test("opens Create Account inside the Admin Shell without losing auth state", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await page.getByRole("link", { name: "Accounts" }).click();
    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page.getByRole("link", { name: "Create Account" }).click();

    await expect(page).toHaveURL(/\/admin\/accounts\/create$/);
    await expect(
      page.getByRole("heading", {
        name: "Create Administrator Account",
      }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: "Accounts" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await expect(
      page.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeVisible();

    await page
      .getByRole("link", {
        name: "Back to Administrator Accounts",
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);
  });

  test("opens Edit Account while preserving list query context", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await page.getByRole("link", { name: "Accounts" }).click();
    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page.evaluate(() => {
      window.history.pushState(
        {},
        "",
        "/admin/accounts?page=2&page_size=50&status=inactive",
      );
      window.dispatchEvent(new PopStateEvent("popstate"));
    });

    await expect(page).toHaveURL(
      /\/admin\/accounts\?page=2&page_size=50&status=inactive$/,
    );
    await expect(
      page.getByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeVisible();

    await page
      .getByRole("link", {
        name: "Edit account: Library Administrator",
      })
      .click();

    await expect(page).toHaveURL(
      /\/admin\/accounts\/01900000-0000-7000-8000-000000000001\/edit\?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D50%26status%3Dinactive$/,
    );
    await expect(
      page.getByRole("heading", {
        name: "Edit Administrator Account",
      }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: "Accounts" })).toHaveAttribute(
      "aria-current",
      "page",
    );

    await page
      .getByRole("link", {
        name: "Back to Administrator Accounts",
      })
      .click();

    await expect(page).toHaveURL(
      /\/admin\/accounts\?page=2&page_size=50&status=inactive$/,
    );
  });
});
