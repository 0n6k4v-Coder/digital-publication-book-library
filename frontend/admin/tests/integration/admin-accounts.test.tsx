import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";
const refreshedAccessToken = "refreshed-access-token";

function problemResponse(status: number, code: string): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/authentication",
      title: "Authentication error",
      status,
      detail: "server detail must not reach the UI",
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

function unauthorizedResponse(): Response {
  return problemResponse(401, "UNAUTHORIZED");
}

function loginResponse(token: string = accessToken): Response {
  return new Response(
    JSON.stringify({
      access_token: token,
      token_type: "Bearer",
      expires_in: 3600,
    }),
    {
      status: 200,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
      },
    },
  );
}

function accountsResponse(): Response {
  return new Response(
    JSON.stringify({
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
    }),
    {
      status: 200,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
      },
    },
  );
}

beforeEach(async () => {
  window.history.replaceState({}, "", "/login");
  authService.clearClientState();
  vi.stubGlobal("fetch", vi.fn());
  vi.mocked(fetch).mockResolvedValueOnce(unauthorizedResponse());
  await authService.retryBootstrap();
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("Admin Accounts integration", () => {
  it("allows an authenticated user to access /admin/accounts", async () => {
    window.history.replaceState({}, "", "/admin/accounts");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountsResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("link", {
        name: /^Accounts$/,
      }),
    ).toHaveAttribute("aria-current", "page");

    expect(
      screen.getByRole("cell", {
        name: "admin@example.com",
      }),
    ).toBeInTheDocument();

    expect(window.location.pathname).toBe("/admin/accounts");

    const accountCall = vi.mocked(fetch).mock.calls[2];

    expect(accountCall[0]).toBe("/admin/accounts?page=1&page_size=20");
    expect(accountCall[1]?.method).toBe("GET");
    expect(new Headers(accountCall[1]?.headers).get("authorization")).toBe(
      `Bearer ${accessToken}`,
    );

    expect(String(accountCall[0])).not.toContain(accessToken);
    expect(document.body).not.toHaveTextContent(accessToken);
  });

  it("redirects unauthenticated access to /admin/accounts to /login", async () => {
    window.history.replaceState({}, "", "/admin/accounts");

    render(<App />);

    await waitFor(() => {
      expect(window.location.pathname).toBe("/login");
    });

    expect(
      screen.getByRole("heading", {
        name: "Sign in",
      }),
    ).toBeInTheDocument();

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(1);
  });

  it("refreshes the access token once after an Accounts API 401 and retries the request", async () => {
    window.history.replaceState({}, "", "/admin/accounts");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(unauthorizedResponse())
      .mockResolvedValueOnce(loginResponse(refreshedAccessToken))
      .mockResolvedValueOnce(accountsResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();

    expect(authService.getAuthorizationHeader()).toBe(
      `Bearer ${refreshedAccessToken}`,
    );

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(4);

    const initialRequest = vi.mocked(fetch).mock.calls[2];
    const refreshRequest = vi.mocked(fetch).mock.calls[3];

    expect(initialRequest[0]).toBe("/admin/accounts?page=1&page_size=20");
    expect(new Headers(initialRequest[1]?.headers).get("authorization")).toBe(
      `Bearer ${accessToken}`,
    );

    expect(refreshRequest[0]).toBe("/admin/accounts?page=1&page_size=20");
    expect(new Headers(refreshRequest[1]?.headers).get("authorization")).toBe(
      `Bearer ${refreshedAccessToken}`,
    );

    const refreshCalls = vi
      .mocked(fetch)
      .mock.calls.filter(([url]) => url === "/auth/refresh");

    expect(refreshCalls).toHaveLength(1);
  });

  it("clears client authentication when the shared refresh returns 401", async () => {
    window.history.replaceState({}, "", "/admin/accounts");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(unauthorizedResponse())
      .mockResolvedValueOnce(unauthorizedResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    await waitFor(() => {
      expect(window.location.pathname).toBe("/login");
    });

    expect(authService.getSnapshot().authStatus).toBe("unauthenticated");
    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(3);
  });

  it("keeps the shell mounted while navigating from /admin to /admin/accounts", async () => {
    window.history.replaceState({}, "", "/admin");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountsResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("link", {
        name: /^Accounts$/,
      }),
    ).toBeInTheDocument();

    await screen.findByRole("button", { name: "Logout" });

    expect(window.location.pathname).toBe("/admin");

    screen
      .getByRole("link", {
        name: /^Accounts$/,
      })
      .click();

    await waitFor(() => {
      expect(window.location.pathname).toBe("/admin/accounts");
    });

    expect(
      screen.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("button", {
        name: "Logout",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();
  });
});
