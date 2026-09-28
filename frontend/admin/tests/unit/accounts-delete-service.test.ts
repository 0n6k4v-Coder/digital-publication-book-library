import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsError, accountsService } from "../../src/services/accounts";
import { authService } from "../../src/services/auth";

vi.mock("../../src/services/auth", () => ({
  authService: {
    fetchWithAuthentication: vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const headers = new Headers(init?.headers);
        headers.set("Authorization", "Bearer opaque-access-token");
        return fetch(input, { ...init, headers });
      },
    ),
  },
}));

const accountId = "01900000-0000-7000-8000-000000000001";

function problemResponse(status: number, code: string): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/account",
      title: "Account deletion error",
      status,
      detail: "internal detail must never be rendered",
      code,
    }),
    {
      status,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/problem+json",
      },
    },
  );
}

beforeEach(() => {
  vi.stubGlobal("fetch", vi.fn());
  vi.mocked(authService.fetchWithAuthentication).mockClear();
});

describe("accountsService delete APIs", () => {
  it("uses DELETE /admin/accounts/{id} and accepts 204", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(new Response(null, { status: 204 }));

    await accountsService.softDelete(accountId);

    const [url, init] = vi.mocked(fetch).mock.calls[0];

    expect(url).toBe(`/admin/accounts/${accountId}`);
    expect(init?.method).toBe("DELETE");
    expect(init?.body).toBeUndefined();
    expect(init?.cache).toBe("no-store");

    expect(new Headers(init?.headers).get("authorization")).toBe(
      "Bearer opaque-access-token",
    );

    expect(String(url)).not.toContain("opaque-access-token");

    expect(JSON.stringify(init?.body ?? "")).not.toContain("password");
  });

  it("uses DELETE /admin/accounts/{id}/purge and accepts 204", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(new Response(null, { status: 204 }));

    await accountsService.hardDelete(accountId);

    const [url, init] = vi.mocked(fetch).mock.calls[0];

    expect(url).toBe(`/admin/accounts/${accountId}/purge`);
    expect(init?.method).toBe("DELETE");
    expect(init?.body).toBeUndefined();
    expect(init?.cache).toBe("no-store");
  });

  it("treats non-204 success responses as invalid", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ ok: true }), {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      }),
    );

    await expect(accountsService.softDelete(accountId)).rejects.toMatchObject({
      code: "INVALID_RESPONSE",
      status: 200,
    });
  });

  it("preserves explicit 403 authorization errors", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(403, "ACCOUNT_DELETE_FORBIDDEN"),
    );

    await expect(accountsService.softDelete(accountId)).rejects.toMatchObject({
      code: "FORBIDDEN",
      status: 403,
      problemCode: "ACCOUNT_DELETE_FORBIDDEN",
    });
  });

  it("preserves the deletion conflict codes", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(409, "ACCOUNT_ALREADY_DELETED"),
    );

    await expect(accountsService.softDelete(accountId)).rejects.toMatchObject({
      code: "CONFLICT",
      status: 409,
      problemCode: "ACCOUNT_ALREADY_DELETED",
    });

    expect(
      String(new AccountsError("CONFLICT", 409, "ACCOUNT_ALREADY_DELETED")),
    ).not.toContain("internal detail");
  });
});
