import { expect, test } from "@playwright/test";

test.describe("authentication bootstrap", () => {
  test("keeps protected routes pending and starts only one bootstrap request under Strict Mode", async ({
    page,
  }) => {
    let refreshRequestCount = 0;
    let releaseBootstrap!: () => void;

    const bootstrapGate = new Promise<void>((resolve) => {
      releaseBootstrap = resolve;
    });

    await page.route("**/auth/refresh", async (route) => {
      refreshRequestCount += 1;

      expect(route.request().method()).toBe("POST");
      expect(route.request().postData()).toBeNull();

      await bootstrapGate;

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

    await expect(page.getByRole("status")).toContainText(
      "Checking authentication…",
    );
    await expect(page).toHaveURL(/\/admin$/);

    releaseBootstrap();

    await expect(page).toHaveURL(/\/login$/);
    expect(refreshRequestCount).toBe(1);
  });
});
