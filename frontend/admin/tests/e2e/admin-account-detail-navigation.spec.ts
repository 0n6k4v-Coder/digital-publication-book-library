import { expect, test, type Page } from "@playwright/test";

const accountId = "01900000-0000-7000-8000-000000000001";

const loginResponse = {
  access_token: "opaque-access-token",
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

const accountListResponse = {
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
  page: 2,
  page_size: 50,
  total: 100,
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

const listUrl =
  "/admin/accounts?page=2&page_size=50&status=active&include_deleted=true";

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
        body: JSON.stringify(accountListResponse),
      });

      return;
    }

    await route.continue();
  });
}

function getAccountsNavigation(page: Page) {
  return page
    .getByRole("navigation", {
      name: "Admin feature navigation",
    })
    .getByRole("link", {
      name: "Accounts",
      exact: true,
    });
}

test.describe("admin account detail navigation", () => {
  test("preserves filtered Account List context through Detail and back navigation", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await getAccountsNavigation(page).click();
    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page.evaluate((href) => {
      window.history.pushState({}, "", href);
      window.dispatchEvent(new PopStateEvent("popstate"));
    }, listUrl);

    await expect(page).toHaveURL(
      /\/admin\/accounts\?page=2&page_size=50&status=active&include_deleted=true$/,
    );

    await page
      .getByRole("link", {
        name: "View account: Library Administrator",
      })
      .click();

    const encodedReturnTo = encodeURIComponent(listUrl);

    await expect(page).toHaveURL(
      new RegExp(
        `/admin/accounts/${accountId}\\?return_to=${encodedReturnTo.replace(
          /[.*+?^${}()|[\]\\]/g,
          "\\$&",
        )}$`,
      ),
    );

    await expect(getAccountsNavigation(page)).toHaveAttribute(
      "aria-current",
      "page",
    );

    await expect(
      page.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", listUrl);

    await expect(
      page.getByRole("link", {
        name: "Edit",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${accountId}/edit?return_to=${encodedReturnTo}`,
    );

    await expect(
      page.getByRole("link", {
        name: "Delete Account",
      }),
    ).toHaveAttribute(
      "href",
      `/admin/accounts/${accountId}/delete?return_to=${encodedReturnTo}`,
    );

    await page
      .getByRole("link", {
        name: "Back to Administrator Accounts",
      })
      .click();

    await expect(page).toHaveURL(
      /\/admin\/accounts\?page=2&page_size=50&status=active&include_deleted=true$/,
    );
  });

  test("navigates from Account Detail to the dedicated Delete page", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountsApi(page);

    await getAccountsNavigation(page).click();
    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page
      .getByRole("link", {
        name: "View account: Library Administrator",
      })
      .click();

    await expect(page).toHaveURL(new RegExp(`/admin/accounts/${accountId}$`));

    await page.getByRole("link", { name: "Delete Account" }).click();

    await expect(page).toHaveURL(
      new RegExp(
        `/admin/accounts/${accountId}/delete\\?return_to=%2Fadmin%2Faccounts$`,
      ),
    );

    await expect(getAccountsNavigation(page)).toHaveAttribute(
      "aria-current",
      "page",
    );
  });
});
