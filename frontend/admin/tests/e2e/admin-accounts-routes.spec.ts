import { expect, test, type Page } from "@playwright/test";

const accountId = "01900000-0000-7000-8000-000000000001";

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
      id: accountId,
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

const accountDetailResponse = {
  id: accountId,
  email: "admin@example.com",
  display_name: "Library Administrator",
  status: "active",
  created_at: "2026-09-23T10:00:00Z",
  updated_at: "2026-09-23T10:00:00Z",
  deleted_at: null,
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
  await page.route("**/admin/accounts**", async (route) => {
    if (route.request().resourceType() !== "fetch") {
      await route.continue();
      return;
    }

    const requestUrl = new URL(route.request().url());

    if (requestUrl.pathname === `/admin/accounts/${accountId}`) {
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
        body: JSON.stringify(accountDetailResponse),
      });

      return;
    }

    if (requestUrl.pathname === "/admin/accounts") {
      expect(route.request().method()).toBe("GET");

      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(accountsResponse),
      });

      return;
    }

    await route.continue();
  });
}

function getAccountsNavigation(page: Page) {
  return page
    .getByRole("navigation", { name: "Admin feature navigation" })
    .getByRole("link", {
      name: "Accounts",
      exact: true,
    });
}

test.describe("admin accounts routes", () => {
  test("opens Create Account inside the Admin Shell without losing auth state", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await getAccountsNavigation(page).click();
    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page.getByRole("link", { name: "Create Account" }).click();

    await expect(page).toHaveURL(/\/admin\/accounts\/create$/);
    await expect(
      page.getByRole("heading", {
        name: "Create Administrator Account",
      }),
    ).toBeVisible();
    await expect(getAccountsNavigation(page)).toHaveAttribute(
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

  test("opens Edit Account inside the Admin Shell and preserves validated list context", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await getAccountsNavigation(page).click();
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

    await page
      .getByRole("link", {
        name: "Edit account: Library Administrator",
      })
      .click();

    await expect(page).toHaveURL(
      new RegExp(
        `/admin/accounts/${accountId}/edit\\?return_to=%2Fadmin%2Faccounts%3Fpage%3D2%26page_size%3D50%26status%3Dinactive$`,
      ),
    );

    await expect(
      page.getByRole("heading", {
        name: "Edit Administrator Account",
      }),
    ).toBeVisible();

    await expect(page.getByLabel("Display name")).toHaveValue(
      "Library Administrator",
    );

    await expect(page.getByText("admin@example.com")).toBeVisible();

    await expect(getAccountsNavigation(page)).toHaveAttribute(
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

    await expect(page).toHaveURL(
      /\/admin\/accounts\?page=2&page_size=50&status=inactive$/,
    );
  });
});
