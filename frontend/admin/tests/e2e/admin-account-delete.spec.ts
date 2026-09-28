import { expect, test } from "@playwright/test";

const accountId = "01900000-0000-7000-8000-000000000001";

const accessToken = "opaque-e2e-access-token";

function accountJson(overrides: Record<string, unknown> = {}) {
  return {
    id: accountId,
    email: "admin@example.com",
    display_name: "Library Administrator",
    status: "active",
    created_at: "2026-09-23T10:00:00Z",
    updated_at: "2026-09-23T10:00:00Z",
    deleted_at: null,
    ...overrides,
  };
}

async function authenticate(
  page: import("@playwright/test").Page,
): Promise<void> {
  await page.route("**/auth/refresh", async (route) => {
    await route.fulfill({
      status: 401,
      headers: {
        "Content-Type": "application/problem+json",
      },
      body: JSON.stringify({
        type: "https://example.invalid/problems/authentication",
        title: "Unauthorized",
        status: 401,
        code: "UNAUTHORIZED",
      }),
    });
  });

  await page.route("**/auth/login", async (route) => {
    await route.fulfill({
      status: 200,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        access_token: accessToken,
        token_type: "Bearer",
        expires_in: 3600,
      }),
    });
  });

  await page.goto("/login");

  await page.getByLabel("Email").fill("admin@example.com");

  await page
    .getByRole("textbox", { name: "Password" })
    .fill("example-secure-password");

  await page
    .getByRole("button", {
      name: "Sign In",
    })
    .click();

  await expect(page).toHaveURL(/\/admin$/);
}

async function navigateToDeletePage(
  page: import("@playwright/test").Page,
  returnTo?: string,
): Promise<void> {
  const destination =
    returnTo === undefined
      ? `/admin/accounts/${accountId}/delete`
      : `/admin/accounts/${accountId}/delete?return_to=${encodeURIComponent(
          returnTo,
        )}`;

  const expectedUrl = new URL(destination, page.url()).toString();

  await page.evaluate((path) => {
    window.history.pushState({}, "", path);
    window.dispatchEvent(new PopStateEvent("popstate"));
  }, destination);

  await expect(page).toHaveURL(expectedUrl);
}

async function stubAccountGet(
  page: import("@playwright/test").Page,
  responseBody: object | null,
  status = 200,
): Promise<void> {
  await page.route(`**/admin/accounts/${accountId}`, async (route) => {
    if (route.request().method() !== "GET") {
      await route.fallback();
      return;
    }

    await route.fulfill({
      status,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/problem+json",
      },
      body:
        responseBody === null
          ? JSON.stringify({
              type: "https://example.invalid/problems/account",
              title: "Not Found",
              status,
              code: "ACCOUNT_NOT_FOUND",
            })
          : JSON.stringify(responseBody),
    });
  });
}

test.describe("Admin Account Delete", () => {
  test("supports soft-delete confirmation and cancel", async ({ page }) => {
    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    await navigateToDeletePage(page, "/admin/accounts");

    await expect(
      page.getByRole("heading", {
        name: "Delete Administrator Account",
      }),
    ).toBeVisible();

    const softTrigger = page.getByRole("button", {
      name: "Soft-delete account",
      exact: true,
    });

    await softTrigger.focus();
    await softTrigger.click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await expect(
      confirmationDialog.getByRole("button", {
        name: "Cancel",
        exact: true,
      }),
    ).toBeFocused();

    await confirmationDialog
      .getByRole("button", {
        name: "Cancel",
        exact: true,
      })
      .click();

    await expect(confirmationDialog).toBeHidden();

    await expect(softTrigger).toBeFocused();
  });

  test("soft-deletes with DELETE /admin/accounts/{id} and 204", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    const calls: string[] = [];

    await page.route(`**/admin/accounts/${accountId}`, async (route) => {
      if (route.request().method() === "GET") {
        await route.fulfill({
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(accountJson()),
        });
        return;
      }

      if (route.request().method() === "DELETE") {
        calls.push(route.request().url());

        expect(route.request().postData()).toBeNull();

        await route.fulfill({
          status: 204,
        });
        return;
      }

      await route.fallback();
    });

    await navigateToDeletePage(page, "/admin/accounts");

    await page
      .getByRole("button", {
        name: "Soft-delete account",
        exact: true,
      })
      .click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await confirmationDialog
      .getByRole("button", {
        name: "Delete Account",
        exact: true,
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);

    expect(calls).toHaveLength(1);

    expect(new URL(calls[0]).pathname).toBe(`/admin/accounts/${accountId}`);
  });

  test("hard-deletes an unavailable/soft-deleted target with the purge endpoint", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountGet(page, null, 404);

    await page.route(`**/admin/accounts/${accountId}/purge`, async (route) => {
      expect(route.request().method()).toBe("DELETE");

      expect(route.request().postData()).toBeNull();

      await route.fulfill({
        status: 204,
      });
    });

    await navigateToDeletePage(page);

    await expect(
      page.getByRole("heading", {
        name: "Account unavailable",
      }),
    ).toBeVisible();

    await page
      .getByRole("button", {
        name: "Permanently delete account",
        exact: true,
      })
      .click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await confirmationDialog
      .getByRole("button", {
        name: "Permanently Delete",
        exact: true,
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);
  });

  test("handles LAST_ACTIVE_ADMINISTRATOR without logout", async ({ page }) => {
    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    await page.route(`**/admin/accounts/${accountId}`, async (route) => {
      if (route.request().method() === "GET") {
        await route.fulfill({
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(accountJson()),
        });
        return;
      }

      await route.fulfill({
        status: 409,
        headers: {
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/last-active-administrator",
          title: "Conflict",
          status: 409,
          code: "LAST_ACTIVE_ADMINISTRATOR",
        }),
      });
    });

    await navigateToDeletePage(page);

    await page
      .getByRole("button", {
        name: "Soft-delete account",
        exact: true,
      })
      .click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await confirmationDialog
      .getByRole("button", {
        name: "Delete Account",
        exact: true,
      })
      .click();

    await expect(
      confirmationDialog.getByRole("alert", {
        exact: true,
      }),
    ).toContainText("The last active administrator cannot be deleted.");

    await expect(
      page.getByRole("heading", {
        name: "Delete Administrator Account",
        exact: true,
      }),
    ).toBeVisible();
  });

  test("Escape cancels confirmation and focus returns to the invoking control", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    await navigateToDeletePage(page);

    const trigger = page.getByRole("button", {
      name: "Soft-delete account",
      exact: true,
    });

    await trigger.focus();
    await trigger.click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await page.keyboard.press("Escape");

    await expect(confirmationDialog).toBeHidden();

    await expect(trigger).toBeFocused();
  });

  test("preserves only supported return_to values and rejects external destinations", async ({
    page,
  }) => {
    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    await page.route(`**/admin/accounts/${accountId}`, async (route) => {
      if (route.request().method() === "GET") {
        await route.fulfill({
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(accountJson()),
        });
        return;
      }

      await route.fulfill({
        status: 204,
      });
    });

    await navigateToDeletePage(page, "https://evil.example/admin/accounts");

    await page
      .getByRole("button", {
        name: "Soft-delete account",
        exact: true,
      })
      .click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await confirmationDialog
      .getByRole("button", {
        name: "Delete Account",
        exact: true,
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);
  });

  test("remains usable on a 390x844 viewport", async ({ page }) => {
    await page.setViewportSize({
      width: 390,
      height: 844,
    });

    await authenticate(page);
    await stubAccountGet(page, accountJson(), 200);

    await navigateToDeletePage(page);

    await expect(
      page.getByRole("button", {
        name: "Soft-delete account",
        exact: true,
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("button", {
        name: "Permanently delete account",
        exact: true,
      }),
    ).toBeVisible();

    await page
      .getByRole("button", {
        name: "Soft-delete account",
        exact: true,
      })
      .click();

    const confirmationDialog = page.getByRole("alertdialog");

    await expect(confirmationDialog).toBeVisible();

    await expect(
      confirmationDialog.getByRole("button", {
        name: "Cancel",
        exact: true,
      }),
    ).toBeVisible();

    await expect(
      confirmationDialog.getByRole("button", {
        name: "Delete Account",
        exact: true,
      }),
    ).toBeVisible();
  });
});
