import { expect, test, type Page, type Route } from "@playwright/test";

const accessToken = "opaque-access-token";
const password = "example-secure-password";
const accountId = "01900000-0000-7000-8000-000000000010";

const loginResponse = {
  access_token: accessToken,
  token_type: "Bearer",
  expires_in: 3600,
  refresh_token: "opaque-refresh-token",
  refresh_expires_in: 2592000,
};

const accountListResponse = {
  items: [],
  page: 1,
  page_size: 20,
  total: 0,
};

const createdAccountResponse = {
  id: accountId,
  email: "created@example.com",
  display_name: null,
  status: "active",
  created_at: "2026-09-27T02:00:00Z",
  updated_at: "2026-09-27T02:00:00Z",
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
  await page.getByRole("textbox", { name: "Password" }).fill(password);
  await page.getByRole("button", { name: "Sign In" }).click();

  await expect(page).toHaveURL(/\/admin$/);
}

function getAccountsNavigation(page: Page) {
  return page
    .getByRole("navigation", { name: "Admin feature navigation" })
    .getByRole("link", {
      name: "Accounts",
      exact: true,
    });
}

async function installAccountsRoutes(
  page: Page,
  handleCreate: (route: Route) => Promise<void>,
): Promise<void> {
  await page.route("**/admin/accounts**", async (route) => {
    if (route.request().resourceType() !== "fetch") {
      await route.continue();
      return;
    }

    const request = route.request();
    const requestUrl = new URL(request.url());

    if (
      request.method() === "GET" &&
      requestUrl.pathname === "/admin/accounts"
    ) {
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

    if (
      request.method() === "GET" &&
      requestUrl.pathname === `/admin/accounts/${accountId}`
    ) {
      await route.fulfill({
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
        body: JSON.stringify(createdAccountResponse),
      });
      return;
    }

    if (
      request.method() === "POST" &&
      requestUrl.pathname === "/admin/accounts"
    ) {
      await handleCreate(route);
      return;
    }

    await route.continue();
  });
}

async function openCreatePage(page: Page): Promise<void> {
  await getAccountsNavigation(page).click();
  await expect(page).toHaveURL(/\/admin\/accounts$/);

  const accountsPageHeader = page.getByRole("banner").filter({
    has: page.getByRole("heading", {
      name: "Administrator Accounts",
      level: 1,
    }),
  });

  await accountsPageHeader
    .getByRole("link", {
      name: "Create Account",
      exact: true,
    })
    .click();

  await expect(page).toHaveURL(/\/admin\/accounts\/create$/);

  await expect(
    page.getByRole("heading", {
      name: "Create Administrator Account",
      level: 1,
    }),
  ).toBeVisible();

  await expect(getAccountsNavigation(page)).toHaveAttribute(
    "aria-current",
    "page",
  );
}

test.describe("admin account create", () => {
  test("renders inside the Admin Shell with Accounts active", async ({
    page,
  }) => {
    await authenticate(page);

    await installAccountsRoutes(page, async (route) => {
      await route.continue();
    });

    await openCreatePage(page);

    await expect(
      page.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeVisible();

    await expect(
      page.getByRole("heading", {
        name: "Create Administrator Account",
        level: 1,
      }),
    ).toBeVisible();

    await expect(getAccountsNavigation(page)).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  test("uses the documented POST /admin/accounts contract and navigates to Account Detail", async ({
    page,
  }) => {
    await authenticate(page);

    await installAccountsRoutes(page, async (route) => {
      const request = route.request();

      expect(request.method()).toBe("POST");
      expect(request.headers().authorization).toBe(`Bearer ${accessToken}`);
      expect(request.headers()["content-type"]).toContain("application/json");

      const requestUrl = request.url();

      expect(requestUrl).not.toContain(accessToken);
      expect(requestUrl).not.toContain(password);

      expect(request.postDataJSON()).toEqual({
        email: "created@example.com",
        password,
      });

      expect(request.postData()).not.toContain("display_name");
      expect(request.postData()).not.toContain(accessToken);

      await route.fulfill({
        status: 201,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
          Location: `/admin/accounts/${accountId}`,
        },
        body: JSON.stringify(createdAccountResponse),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("created@example.com");

    await page.getByLabel("Password").fill(password);

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    await expect(page).toHaveURL(new RegExp(`/admin/accounts/${accountId}$`));

    await expect(
      page.getByRole("heading", {
        name: "Administrator Account",
        level: 1,
      }),
    ).toBeVisible();

    const storage = await page.evaluate(() => ({
      localStorage: JSON.stringify(localStorage),
      sessionStorage: JSON.stringify(sessionStorage),
    }));

    expect(JSON.stringify(storage)).not.toContain(password);
    expect(page.url()).not.toContain(accessToken);
    expect(page.url()).not.toContain(password);
    await expect(page.getByText(accessToken, { exact: false })).toHaveCount(0);
  });

  test("uses native validation and does not submit an invalid empty form", async ({
    page,
  }) => {
    await authenticate(page);

    let postCount = 0;

    await installAccountsRoutes(page, async (route) => {
      postCount += 1;
      await route.continue();
    });

    await openCreatePage(page);

    const email = page.getByLabel("Email");
    const passwordInput = page.getByLabel("Password");

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    const emailIsInvalid = await email.evaluate(
      (element) => !(element as HTMLInputElement).checkValidity(),
    );
    const passwordIsInvalid = await passwordInput.evaluate(
      (element) => !(element as HTMLInputElement).checkValidity(),
    );

    expect(emailIsInvalid).toBe(true);
    expect(passwordIsInvalid).toBe(true);

    expect(postCount).toBe(0);

    const isValid = await page
      .getByRole("form", {
        name: "Account credentials",
      })
      .evaluate((form) => (form as HTMLFormElement).checkValidity());

    expect(isValid).toBe(false);
  });

  test("keeps the form retryable after EMAIL_ALREADY_IN_USE", async ({
    page,
  }) => {
    await authenticate(page);

    await installAccountsRoutes(page, async (route) => {
      await route.fulfill({
        status: 409,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/account-email-conflict",
          title: "Conflict",
          status: 409,
          detail: "internal diagnostic must not be rendered",
          code: "EMAIL_ALREADY_IN_USE",
        }),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("existing@example.com");

    await page.getByLabel("Password").fill(password);

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    await expect(
      page.getByText("This email address is already in use."),
    ).toBeVisible();

    await expect(page.getByLabel("Email")).toHaveAttribute(
      "aria-invalid",
      "true",
    );

    await expect(page.getByLabel("Email")).toHaveValue("existing@example.com");

    await expect(page.getByLabel("Password")).toHaveValue(password);

    await expect(
      page.getByText("internal diagnostic must not be rendered", {
        exact: false,
      }),
    ).toHaveCount(0);

    await expect(
      page.getByRole("button", {
        name: "Create Account",
      }),
    ).toBeEnabled();
  });

  test("renders 422 validation feedback without internal server diagnostics", async ({
    page,
  }) => {
    await authenticate(page);

    await installAccountsRoutes(page, async (route) => {
      await route.fulfill({
        status: 422,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/account-validation",
          title: "Validation error",
          status: 422,
          detail: "internal validation implementation detail",
          code: "VALIDATION_ERROR",
        }),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("created@example.com");

    await page.getByLabel("Password").fill(password);

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    await expect(page.getByRole("alert")).toHaveText(
      "The server rejected the account details. Review the fields and try again.",
    );

    await expect(
      page.getByText("internal validation implementation detail", {
        exact: false,
      }),
    ).toHaveCount(0);

    await expect(
      page.getByRole("button", {
        name: "Create Account",
      }),
    ).toBeEnabled();
  });

  test("renders 403 as an authorization error without redirecting to login", async ({
    page,
  }) => {
    await authenticate(page);

    await installAccountsRoutes(page, async (route) => {
      await route.fulfill({
        status: 403,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/account-create-forbidden",
          title: "Forbidden",
          status: 403,
          detail: "role mapping must not be exposed",
          code: "ACCOUNT_CREATE_FORBIDDEN",
        }),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("created@example.com");

    await page.getByLabel("Password").fill(password);

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    await expect(page.getByRole("alert")).toHaveText(
      "You do not have permission to create administrator accounts.",
    );

    await expect(page).toHaveURL(/\/admin\/accounts\/create$/);
    await expect(getAccountsNavigation(page)).toHaveAttribute(
      "aria-current",
      "page",
    );

    await expect(
      page.getByRole("button", {
        name: "Logout",
      }),
    ).toBeVisible();

    await expect(
      page.getByText("role mapping must not be exposed", {
        exact: false,
      }),
    ).toHaveCount(0);
  });

  test("delegates 401 to shared authentication behavior", async ({ page }) => {
    await authenticate(page);

    await page.route("**/auth/refresh", async (route) => {
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/unauthorized",
          title: "Unauthorized",
          status: 401,
          code: "UNAUTHORIZED",
        }),
      });
    });

    await installAccountsRoutes(page, async (route) => {
      await route.fulfill({
        status: 401,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/problem+json",
        },
        body: JSON.stringify({
          type: "https://example.invalid/problems/unauthorized",
          title: "Unauthorized",
          status: 401,
          code: "UNAUTHORIZED",
        }),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("created@example.com");

    await page.getByLabel("Password").fill(password);

    await page
      .getByRole("button", {
        name: "Create Account",
      })
      .click();

    await expect(page).toHaveURL(/\/login$/);

    await expect(
      page.getByRole("heading", {
        name: "Sign in",
      }),
    ).toBeVisible();
  });

  test("prevents duplicate browser submissions while the first request is pending", async ({
    page,
  }) => {
    await authenticate(page);

    let postCount = 0;
    let releaseCreate!: () => void;

    await installAccountsRoutes(page, async (route) => {
      postCount += 1;

      await new Promise<void>((resolve) => {
        releaseCreate = resolve;
      });

      await route.fulfill({
        status: 201,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
          Location: `/admin/accounts/${accountId}`,
        },
        body: JSON.stringify(createdAccountResponse),
      });
    });

    await openCreatePage(page);

    await page.getByLabel("Email").fill("created@example.com");

    await page.getByLabel("Password").fill(password);

    const submitButton = page.getByRole("button", {
      name: "Create Account",
    });

    await submitButton.click();

    await expect(
      page.getByRole("button", {
        name: "Creating…",
      }),
    ).toBeDisabled();

    await page
      .getByRole("form", {
        name: "Account credentials",
      })
      .evaluate((form) => {
        (form as HTMLFormElement).requestSubmit();
      });

    await expect.poll(() => postCount).toBe(1);

    releaseCreate();

    await expect(page).toHaveURL(new RegExp(`/admin/accounts/${accountId}$`));
  });

  test("Back and Cancel both return to /admin/accounts without submitting", async ({
    page,
  }) => {
    await authenticate(page);

    let postCount = 0;

    await installAccountsRoutes(page, async (route) => {
      if (route.request().method() === "POST") {
        postCount += 1;
      }

      await route.continue();
    });

    await openCreatePage(page);

    await page
      .getByRole("link", {
        name: "Back to Administrator Accounts",
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);

    await page
      .getByRole("link", {
        name: "Create Account",
        exact: true,
      })
      .click();

    await expect(page).toHaveURL(/\/admin\/accounts\/create$/);

    await page.getByRole("link", { name: "Cancel" }).click();

    await expect(page).toHaveURL(/\/admin\/accounts$/);
    expect(postCount).toBe(0);
  });
});
