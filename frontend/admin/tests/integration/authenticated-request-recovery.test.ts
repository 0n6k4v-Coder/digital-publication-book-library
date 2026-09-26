import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { accountsService } from "../../src/services/accounts";
import { authService } from "../../src/services/auth";

const initialAccessToken = "opaque-access-token";
const refreshedAccessToken = "refreshed-access-token";

function response(
  body: unknown,
  status = 200,
  contentType = "application/json",
): Response {
  return new Response(
    contentType === "application/json"
      ? JSON.stringify(body)
      : body === null
        ? null
        : String(body),
    {
      status,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": contentType,
      },
    },
  );
}

function unauthorizedResponse(): Response {
  return response(
    {
      status: 401,
      code: "UNAUTHORIZED",
    },
    401,
    "application/problem+json",
  );
}

function serverErrorResponse(): Response {
  return response(
    {
      status: 500,
      code: "UNKNOWN",
    },
    500,
    "application/problem+json",
  );
}

function loginResponse(token: string): Response {
  return response({
    access_token: token,
    token_type: "Bearer",
    expires_in: 3600,
  });
}

function accountsResponse(): Response {
  return response({
    items: [
      {
        id: "01900000-0000-7000-8000-000000000001",
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
  });
}

async function authenticate(): Promise<void> {
  await authService.login({
    email: "admin@example.com",
    password: "example-secure-password",
  });
}

beforeEach(async () => {
  authService.clearClientState();

  vi.stubGlobal("fetch", vi.fn());

  vi.mocked(fetch).mockResolvedValueOnce(unauthorizedResponse());

  await authService.retryBootstrap();

  vi.mocked(fetch).mockResolvedValueOnce(loginResponse(initialAccessToken));

  await authenticate();

  vi.mocked(fetch).mockClear();
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("protected-request authentication recovery", () => {
  it("shares one refresh operation across concurrent 401 responses", async () => {
    let accountRequestCount = 0;
    let resolveRefresh!: (response: Response) => void;

    const refreshStarted = new Promise<void>((resolve) => {
      vi.mocked(fetch).mockImplementation(async (input, init) => {
        const url = String(input);

        if (url === "/admin/accounts?page=1&page_size=20") {
          accountRequestCount += 1;

          if (accountRequestCount <= 2) {
            return unauthorizedResponse();
          }

          expect(new Headers(init?.headers).get("authorization")).toBe(
            `Bearer ${refreshedAccessToken}`,
          );

          return accountsResponse();
        }

        if (url === "/auth/refresh") {
          expect(init?.method).toBe("POST");
          expect(init?.credentials).toBe("include");
          expect(init?.body).toBeUndefined();
          expect(new Headers(init?.headers).get("authorization")).toBeNull();

          resolve();

          return new Promise<Response>((refreshResolve) => {
            resolveRefresh = refreshResolve;
          });
        }

        throw new Error(`Unexpected fetch URL: ${url}`);
      });
    });

    const first = accountsService.list();
    const second = accountsService.list();

    await refreshStarted;

    const recoveryRefreshCalls = vi
      .mocked(fetch)
      .mock.calls.filter(([url]) => url === "/auth/refresh");

    expect(recoveryRefreshCalls).toHaveLength(1);

    resolveRefresh(loginResponse(refreshedAccessToken));

    const [firstResult, secondResult] = await Promise.all([first, second]);

    expect(firstResult.total).toBe(1);
    expect(secondResult.total).toBe(1);
    expect(accountRequestCount).toBe(4);
    expect(authService.getAuthorizationHeader()).toBe(
      `Bearer ${refreshedAccessToken}`,
    );
    expect(authService.getSnapshot().authStatus).toBe("authenticated");
  });

  it("reuses an access token refreshed by another concurrent request instead of refreshing again", async () => {
    let accountRequestCount = 0;
    let resolveSecondInitialRequest!: (response: Response) => void;

    const refreshStarted = new Promise<void>((resolve) => {
      vi.mocked(fetch).mockImplementation(async (input, init) => {
        const url = String(input);

        if (url === "/admin/accounts?page=1&page_size=20") {
          accountRequestCount += 1;

          if (accountRequestCount === 1) {
            return unauthorizedResponse();
          }

          if (accountRequestCount === 2) {
            return new Promise<Response>((requestResolve) => {
              resolveSecondInitialRequest = requestResolve;
            });
          }

          expect(new Headers(init?.headers).get("authorization")).toBe(
            `Bearer ${refreshedAccessToken}`,
          );

          return accountsResponse();
        }

        if (url === "/auth/refresh") {
          resolve();
          return loginResponse(refreshedAccessToken);
        }

        throw new Error(`Unexpected fetch URL: ${url}`);
      });
    });

    const first = accountsService.list();
    const second = accountsService.list();

    await refreshStarted;

    const refreshCallsBeforeSecond401 = vi
      .mocked(fetch)
      .mock.calls.filter(([url]) => url === "/auth/refresh");

    expect(refreshCallsBeforeSecond401).toHaveLength(1);

    resolveSecondInitialRequest(unauthorizedResponse());

    const [firstResult, secondResult] = await Promise.all([first, second]);

    expect(firstResult.total).toBe(1);
    expect(secondResult.total).toBe(1);
    expect(
      vi.mocked(fetch).mock.calls.filter(([url]) => url === "/auth/refresh"),
    ).toHaveLength(1);
    expect(authService.getAuthorizationHeader()).toBe(
      `Bearer ${refreshedAccessToken}`,
    );
  });

  it("does not retry when the shared refresh returns 401", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(unauthorizedResponse())
      .mockResolvedValueOnce(unauthorizedResponse());

    await expect(accountsService.list()).rejects.toMatchObject({
      code: "UNAUTHORIZED",
      status: 401,
    });

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(2);
    expect(vi.mocked(fetch).mock.calls[0][0]).toBe(
      "/admin/accounts?page=1&page_size=20",
    );
    expect(vi.mocked(fetch).mock.calls[1][0]).toBe("/auth/refresh");

    expect(authService.getSnapshot().authStatus).toBe("unauthenticated");
  });

  it("does not retry a failed refresh with a second refresh attempt", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(unauthorizedResponse())
      .mockResolvedValueOnce(serverErrorResponse());

    await expect(accountsService.list()).rejects.toMatchObject({
      code: "SERVER",
      status: 500,
    });

    const refreshCalls = vi
      .mocked(fetch)
      .mock.calls.filter(([url]) => url === "/auth/refresh");

    expect(refreshCalls).toHaveLength(1);
    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(2);
    expect(authService.getSnapshot().authStatus).toBe("authentication-error");
  });

  it("retries an affected request at most once", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(unauthorizedResponse())
      .mockResolvedValueOnce(loginResponse(refreshedAccessToken))
      .mockResolvedValueOnce(unauthorizedResponse());

    await expect(accountsService.list()).rejects.toMatchObject({
      code: "UNAUTHORIZED",
      status: 401,
    });

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(3);

    const refreshCalls = vi
      .mocked(fetch)
      .mock.calls.filter(([url]) => url === "/auth/refresh");

    expect(refreshCalls).toHaveLength(1);
    expect(authService.getSnapshot().authStatus).toBe("unauthenticated");
  });

  it("does not recursively refresh /auth/refresh", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(unauthorizedResponse());

    const result = await authService.fetchWithAuthentication("/auth/refresh", {
      method: "POST",
      credentials: "include",
    });

    expect(result.status).toBe(401);
    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(fetch).mock.calls[0][0]).toBe("/auth/refresh");
  });
});
