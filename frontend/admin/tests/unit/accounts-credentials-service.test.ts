import { beforeEach, describe, expect, it, vi } from "vitest";
import { AccountsError, accountsService } from "../../src/services/accounts";
import { authService } from "../../src/services/auth";

vi.mock("../../src/services/auth", () => ({
  authService: {
    fetchWithAuthentication: vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const headers = new Headers(init?.headers);
        headers.set("Authorization", "Bearer opaque-access-token");

        return fetch(input, {
          ...init,
          headers,
        });
      },
    ),
  },
}));

const accountId = "01900000-0000-7000-8000-000000000001";

const accountResponse = {
  id: accountId,
  email: "new-admin@example.com",
  display_name: "Library Administrator",
  status: "active",
  created_at: "2026-09-23T10:00:00Z",
  updated_at: "2026-09-23T10:00:00Z",
  deleted_at: null,
};

beforeEach(() => {
  vi.stubGlobal("fetch", vi.fn());
  vi.mocked(authService.fetchWithAuthentication).mockClear();
});

describe("accountsService credential mutations", () => {
  it("changes email with the documented JSON contract and returns the server Account", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify(accountResponse), {
        status: 200,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
        },
      }),
    );

    const result = await accountsService.changeEmail(
      accountId,
      "new-admin@example.com",
    );

    expect(result.email).toBe("new-admin@example.com");

    const [url, init] = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);

    expect(url).toBe(`/admin/accounts/${accountId}/email`);
    expect(init?.method).toBe("PATCH");
    expect(init?.cache).toBe("no-store");
    expect(headers.get("authorization")).toBe("Bearer opaque-access-token");
    expect(headers.get("content-type")).toBe("application/json");
    expect(init?.body).toBe(
      JSON.stringify({
        email: "new-admin@example.com",
      }),
    );
    expect(String(url)).not.toContain("opaque-access-token");
  });

  it("maps EMAIL_ALREADY_IN_USE without leaking server diagnostics", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          type: "https://example.invalid/problems/email-already-in-use",
          title: "Email already in use",
          status: 409,
          detail: "internal detail must not be rendered",
          code: "EMAIL_ALREADY_IN_USE",
        }),
        {
          status: 409,
          headers: {
            "Content-Type": "application/problem+json",
          },
        },
      ),
    );

    const error = await accountsService
      .changeEmail(accountId, "taken@example.com")
      .catch((value: unknown) => value as AccountsError);

    expect(error).toBeInstanceOf(AccountsError);
    expect(error).toMatchObject({
      code: "CONFLICT",
      status: 409,
      problemCode: "EMAIL_ALREADY_IN_USE",
    });
    expect(String(error)).not.toContain("internal detail");
  });

  it("changes password with only the password payload and accepts 204 No Content", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(null, {
        status: 204,
        headers: {
          "Cache-Control": "no-store",
        },
      }),
    );

    await expect(
      accountsService.changePassword(accountId, "very-long-example-password"),
    ).resolves.toBeUndefined();

    const [url, init] = vi.mocked(fetch).mock.calls[0];

    expect(url).toBe(`/admin/accounts/${accountId}/password`);
    expect(init?.method).toBe("PATCH");
    expect(init?.cache).toBe("no-store");
    expect(new Headers(init?.headers).get("content-type")).toBe(
      "application/json",
    );
    expect(init?.body).toBe(
      JSON.stringify({
        password: "very-long-example-password",
      }),
    );
    expect(String(url)).not.toContain("very-long-example-password");
  });

  it("maps PASSWORD_POLICY_VIOLATION", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          type: "https://example.invalid/problems/password-policy",
          title: "Password policy violation",
          status: 422,
          code: "PASSWORD_POLICY_VIOLATION",
        }),
        {
          status: 422,
          headers: {
            "Content-Type": "application/problem+json",
          },
        },
      ),
    );

    const error = await accountsService
      .changePassword(accountId, "short")
      .catch((value: unknown) => value as AccountsError);

    expect(error).toBeInstanceOf(AccountsError);
    expect(error).toMatchObject({
      code: "VALIDATION",
      status: 422,
      problemCode: "PASSWORD_POLICY_VIOLATION",
    });
  });
});
