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

const accountId = "01900000-0000-7000-8000-000000000010";

const createdAccountResponse = {
  id: accountId,
  email: "created@example.com",
  display_name: null,
  status: "active",
  created_at: "2026-09-27T02:00:00Z",
  updated_at: "2026-09-27T02:00:00Z",
  deleted_at: null,
};

function problemResponse(
  status: number,
  code: string,
  detail = "internal server detail must not reach the UI",
): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/account-create",
      title: "Account create failure",
      status,
      detail,
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

describe("accountsService.create", () => {
  it("sends exactly the supported Account Create request", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify(createdAccountResponse), {
        status: 201,
        headers: {
          "Cache-Control": "no-store",
          "Content-Type": "application/json",
          Location: `/admin/accounts/${accountId}`,
        },
      }),
    );

    const account = await accountsService.create({
      email: "created@example.com",
      password: "example-secure-password",
    });

    expect(
      vi.mocked(authService.fetchWithAuthentication),
    ).toHaveBeenCalledTimes(1);

    const [url, init] = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(init?.headers);

    expect(url).toBe("/admin/accounts");
    expect(init?.method).toBe("POST");
    expect(init?.cache).toBe("no-store");
    expect(headers.get("authorization")).toBe("Bearer opaque-access-token");
    expect(headers.get("content-type")).toBe("application/json");
    expect(init?.body).toBe(
      JSON.stringify({
        email: "created@example.com",
        password: "example-secure-password",
      }),
    );

    expect(String(url)).not.toContain("opaque-access-token");
    expect(String(url)).not.toContain("example-secure-password");
    expect(String(init?.body)).not.toContain("opaque-access-token");
    expect(String(init?.body)).not.toContain("display_name");

    expect(account.id).toBe(accountId);
    expect(account.email).toBe("created@example.com");
    expect(account).not.toHaveProperty("password");
    expect(account).not.toHaveProperty("password_hash");
  });

  it("maps EMAIL_ALREADY_IN_USE to a conflict without exposing Problem Details detail", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(409, "EMAIL_ALREADY_IN_USE"),
    );

    const error = await accountsService
      .create({
        email: "existing@example.com",
        password: "example-secure-password",
      })
      .catch((value: unknown) => value as AccountsError);

    expect(error).toBeInstanceOf(AccountsError);
    expect(error).toMatchObject({
      code: "CONFLICT",
      status: 409,
      problemCode: "EMAIL_ALREADY_IN_USE",
    });
    expect(String(error)).not.toContain("internal server detail");
  });

  it("maps VALIDATION_ERROR from 422", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(422, "VALIDATION_ERROR"),
    );

    await expect(
      accountsService.create({
        email: "created@example.com",
        password: "example-secure-password",
      }),
    ).rejects.toMatchObject({
      code: "VALIDATION",
      status: 422,
      problemCode: "VALIDATION_ERROR",
    });
  });

  it("maps 403 to authorization failure without changing authentication state", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(403, "ACCOUNT_CREATE_FORBIDDEN"),
    );

    await expect(
      accountsService.create({
        email: "created@example.com",
        password: "example-secure-password",
      }),
    ).rejects.toMatchObject({
      code: "FORBIDDEN",
      status: 403,
      problemCode: "ACCOUNT_CREATE_FORBIDDEN",
    });
  });

  it("maps 401 to the shared authentication boundary", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(401, "UNAUTHORIZED"),
    );

    await expect(
      accountsService.create({
        email: "created@example.com",
        password: "example-secure-password",
      }),
    ).rejects.toMatchObject({
      code: "UNAUTHORIZED",
      status: 401,
      problemCode: "UNAUTHORIZED",
    });
  });
});
