import { expect, test, type Page } from "@playwright/test";

const accountId = "01900000-0000-7000-8000-000000000001";

const loginResponse = {
  access_token: "opaque-access-token",
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

function accountResponse(
  overrides: Partial<{
    email: string;
    display_name: string | null;
    updated_at: string;
  }> = {},
) {
  return {
    id: accountId,
    email: overrides.email ?? "admin@example.com",
    display_name: overrides.display_name ?? "Library Administrator",
    status: "active",
    created_at: "2026-09-23T10:00:00Z",
    updated_at: overrides.updated_at ?? "2026-09-23T10:00:00Z",
    deleted_at: null,
  };
}

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

  await page.route("**/auth/refresh", async (route) => {
    expect(route.request().method()).toBe("POST");
    expect(route.request().postData()).toBeNull();

    await route.fulfill({
      status: 200,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
      },
      body: JSON.stringify(loginResponse),
    });
  });
}

async function stubAccountEditApi(page: Page): Promise<void> {
  await page.route("**/admin/accounts**", async (route) => {
    if (route.request().resourceType() !== "fetch") {
      await route.continue();
      return;
    }

    const requestUrl = new URL(route.request().url());

    if (
      requestUrl.pathname === `/admin/accounts/${accountId}` &&
      route.request().method() === "GET"
    ) {
      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(accountResponse()),
      });

      return;
    }

    if (
      requestUrl.pathname === `/admin/accounts/${accountId}` &&
      route.request().method() === "PATCH"
    ) {
      const body = route.request().postDataJSON();

      expect(body).toEqual({
        display_name: "Canonical Display Name",
      });

      expect(route.request().url()).not.toContain("opaque-access-token");

      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(
          accountResponse({
            display_name: "Canonical Display Name",
            updated_at: "2026-09-28T11:00:00Z",
          }),
        ),
      });

      return;
    }

    if (
      requestUrl.pathname === `/admin/accounts/${accountId}/email` &&
      route.request().method() === "PATCH"
    ) {
      const body = route.request().postDataJSON();

      if (body.email === "taken@example.com") {
        await route.fulfill({
          status: 409,
          headers: {
            "Content-Type": "application/problem+json",
          },
          body: JSON.stringify({
            type: "https://example.invalid/problems/email-already-in-use",
            title: "Email already in use",
            status: 409,
            code: "EMAIL_ALREADY_IN_USE",
          }),
        });

        return;
      }

      expect(body).toEqual({
        email: "new-admin@example.com",
      });

      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(
          accountResponse({
            email: "new-admin@example.com",
          }),
        ),
      });

      return;
    }

    if (
      requestUrl.pathname === `/admin/accounts/${accountId}/password` &&
      route.request().method() === "PATCH"
    ) {
      const body = route.request().postDataJSON();

      expect(typeof body.password).toBe("string");
      expect(body.password.length).toBeGreaterThanOrEqual(15);
      expect(route.request().url()).not.toContain(body.password);

      await route.fulfill({
        status: 204,
        headers: {
          "Cache-Control": "no-store",
        },
      });

      return;
    }

    await route.continue();
  });
}

test.describe("admin account edit", () => {
  test("renders inside the Admin Shell and excludes lifecycle actions", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    const returnTo = "/admin/accounts?page=2&page_size=50&status=active";

    await page.goto(
      `/admin/accounts/${accountId}/edit?return_to=${encodeURIComponent(
        returnTo,
      )}`,
    );

    await expect(
      page.getByRole("heading", {
        name: "Edit Administrator Account",
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("navigation", {
        name: "Admin feature navigation",
      }),
    ).toBeVisible();

    await expect(page.getByLabel("Display name")).toHaveValue(
      "Library Administrator",
    );

    await expect(
      page.getByRole("button", {
        name: /deactivate|activate|restore|delete/i,
      }),
    ).toHaveCount(0);

    await expect(
      page.getByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", returnTo);

    await expect(page).toHaveURL(
      new RegExp(`/admin/accounts/${accountId}/edit\\?return_to=`),
    );
  });

  test("updates display name using the dedicated Account PATCH", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    await page.goto(`/admin/accounts/${accountId}/edit`);

    const input = page.getByLabel("Display name");

    await input.fill("Canonical Display Name");

    await page
      .getByRole("button", {
        name: "Save changes",
      })
      .click();

    await expect(input).toHaveValue("Canonical Display Name");

    await expect(page.getByRole("status")).toContainText(
      "Account changes saved.",
    );
  });

  test("handles EMAIL_ALREADY_IN_USE explicitly", async ({ page }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    await page.goto(`/admin/accounts/${accountId}/edit`);

    await page.getByLabel("New email").fill("taken@example.com");

    await page
      .getByRole("button", {
        name: "Change email",
      })
      .click();

    await expect(
      page.getByText("That email address is already in use."),
    ).toBeVisible();

    await expect(page.getByLabel("New email")).toHaveAttribute(
      "aria-invalid",
      "true",
    );
  });

  test("changes email and uses the returned Account representation", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    await page.goto(`/admin/accounts/${accountId}/edit`);

    await page.getByLabel("New email").fill("new-admin@example.com");

    await page
      .getByRole("button", {
        name: "Change email",
      })
      .click();

    await expect(
      page.getByText("new-admin@example.com", {
        exact: true,
      }),
    ).toHaveCount(2);

    await expect(page.getByLabel("New email")).toHaveValue("");

    await expect(page.getByRole("status")).toContainText(
      "Email address changed.",
    );
  });

  test("enforces the client password minimum and clears the field on 204", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    await page.goto(`/admin/accounts/${accountId}/edit`);

    const password = page.getByLabel("New password");
    const button = page.getByRole("button", {
      name: "Change password",
    });

    await password.fill("12345678901234");

    await expect(button).toBeDisabled();

    await password.fill("123456789012345");

    await expect(button).toBeEnabled();

    await button.click();

    await expect(password).toHaveValue("");

    await expect(page.getByRole("status")).toContainText(
      "Password changed successfully.",
    );
  });

  test("uses native password semantics and accessible labels", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountEditApi(page);

    await page.goto(`/admin/accounts/${accountId}/edit`);

    const password = page.getByLabel("New password");

    await expect(password).toHaveAttribute("type", "password");
    await expect(password).toHaveAttribute("autocomplete", "new-password");

    const email = page.getByLabel("New email");
    const emailForm = email.locator("xpath=ancestor::form");

    await expect(email).toHaveAttribute("type", "email");
    await expect(emailForm).toHaveAttribute("novalidate", "");
  });
});
