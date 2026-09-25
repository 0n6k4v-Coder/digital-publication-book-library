import { beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_ACCOUNT_LIST_QUERY } from "../../src/types/account-query";
import { AccountsError, accountsService } from "../../src/services/accounts";
import { authService } from "../../src/services/auth";

vi.mock("../../src/services/auth", () => ({
  authService: {
    getAuthorizationHeader: vi.fn(() => "Bearer opaque-access-token"),
    clearClientState: vi.fn(),
  },
}));

const accountResponse = {
  items: [
    {
      id: "01900000-0000-7000-8000-000000000001",
      email: "admin@example.com",
      display_name: null,
      status: "active",
      created_at: "2026-09-23T10:00:00Z",
      updated_at: "2026-09-23T10:00:00Z",
      deleted_at: null,
      password: "must-never-be-consumed",
      password_hash: "must-never-be-consumed",
    },
  ],
  page: 2,
  page_size: 100,
  total: 101,
};

beforeEach(() => {
  vi.stubGlobal("fetch", vi.fn());
  vi.mocked(authService.getAuthorizationHeader).mockReturnValue(
    "Bearer opaque-access-token",
  );
  vi.mocked(authService.clearClientState).mockClear();
});

describe("accountsService", () => {
  it("sends the documented GET query and authorization header", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify(accountResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );

    const data = await accountsService.list({
      page: 2,
      pageSize: 100,
      status: "inactive",
      includeDeleted: true,
    });

    expect(fetch).toHaveBeenCalledTimes(1);
    const [url, init] = vi.mocked(fetch).mock.calls[0];

    expect(url).toBe(
      "/admin/accounts?page=2&page_size=100&status=inactive&include_deleted=true",
    );
    expect(init?.method).toBe("GET");
    expect(new Headers(init?.headers).get("authorization")).toBe(
      "Bearer opaque-access-token",
    );
    expect(init?.body).toBeUndefined();
    expect(init?.cache).toBe("no-store");
    expect(String(url)).not.toContain("opaque-access-token");
    expect(data.items[0]).not.toHaveProperty("password");
    expect(data.items[0]).not.toHaveProperty("password_hash");
  });

  it("uses the documented default API values", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          items: [],
          page: 1,
          page_size: 20,
          total: 0,
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      ),
    );

    await accountsService.list(DEFAULT_ACCOUNT_LIST_QUERY);

    expect(vi.mocked(fetch).mock.calls[0][0]).toBe(
      "/admin/accounts?page=1&page_size=20",
    );
  });

  it("clears auth state on 401 and preserves it on 403", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            type: "https://example.invalid/problems/unauthorized",
            title: "Unauthorized",
            status: 401,
            code: "UNAUTHORIZED",
          }),
          {
            status: 401,
            headers: { "Content-Type": "application/problem+json" },
          },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            type: "https://example.invalid/problems/forbidden",
            title: "Forbidden",
            status: 403,
            code: "ACCOUNT_VIEW_FORBIDDEN",
          }),
          {
            status: 403,
            headers: { "Content-Type": "application/problem+json" },
          },
        ),
      );

    await expect(accountsService.list()).rejects.toMatchObject({
      code: "UNAUTHORIZED",
      status: 401,
      problemCode: "UNAUTHORIZED",
    });
    expect(authService.clearClientState).toHaveBeenCalledTimes(1);

    await expect(accountsService.list()).rejects.toMatchObject({
      code: "FORBIDDEN",
      status: 403,
      problemCode: "ACCOUNT_VIEW_FORBIDDEN",
    });
    expect(authService.clearClientState).toHaveBeenCalledTimes(1);
  });

  it("maps Problem Details conflicts without exposing server detail", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          type: "https://example.invalid/problems/last-active-administrator",
          title: "Conflict",
          status: 409,
          detail: "internal database detail must not be rendered",
          code: "LAST_ACTIVE_ADMINISTRATOR",
        }),
        {
          status: 409,
          headers: { "Content-Type": "application/problem+json" },
        },
      ),
    );

    const error = await accountsService
      .mutate("deactivate", "01900000-0000-7000-8000-000000000001")
      .catch((value: unknown) => value as AccountsError);

    expect(error).toBeInstanceOf(AccountsError);
    expect(error).toMatchObject({
      code: "CONFLICT",
      status: 409,
      problemCode: "LAST_ACTIVE_ADMINISTRATOR",
    });
    expect(String(error)).not.toContain("internal database detail");
  });

  it("uses POST mutations without token bodies or tokenized URLs", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(new Response(null, { status: 200 }));

    await accountsService.mutate(
      "restore",
      "01900000-0000-7000-8000-000000000001",
    );

    const [url, init] = vi.mocked(fetch).mock.calls[0];
    expect(url).toBe(
      "/admin/accounts/01900000-0000-7000-8000-000000000001/restore",
    );
    expect(init?.method).toBe("POST");
    expect(init?.body).toBeUndefined();
    expect(String(url)).not.toContain("opaque-access-token");
  });
});
