import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";
const refreshToken = "opaque-refresh-token";

function loginResponse(): Response {
  return new Response(
    JSON.stringify({
      access_token: accessToken,
      token_type: "Bearer",
      expires_in: 3600,
      refresh_token: refreshToken,
      refresh_expires_in: 2592000,
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

function unauthorizedResponse(): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/unauthorized",
      title: "Unauthorized",
      status: 401,
      detail: "authentication is no longer valid",
      code: "UNAUTHORIZED",
    }),
    {
      status: 401,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/problem+json",
      },
    },
  );
}

beforeEach(() => {
  window.history.replaceState({}, "", "/login");
  authService.clearClientState();
  vi.stubGlobal("fetch", vi.fn());
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
        name: "Accounts",
        exact: true,
      }),
    ).toHaveAttribute("aria-current", "page");

    expect(
      screen.getByRole("cell", {
        name: "admin@example.com",
      }),
    ).toBeInTheDocument();

    expect(window.location.pathname).toBe("/admin/accounts");

    const accountCall = vi.mocked(fetch).mock.calls[1];

    expect(accountCall[0]).toBe("/admin/accounts?page=1&page_size=20");
    expect(accountCall[1]?.method).toBe("GET");
    expect(new Headers(accountCall[1]?.headers).get("authorization")).toBe(
      `Bearer ${accessToken}`,
    );

    expect(String(accountCall[0])).not.toContain(accessToken);
    expect(document.body).not.toHaveTextContent(accessToken);
    expect(document.body).not.toHaveTextContent(refreshToken);
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

    expect(vi.mocked(fetch)).not.toHaveBeenCalled();
  });

  it("clears client authentication after an Accounts API 401", async () => {
    window.history.replaceState({}, "", "/admin/accounts");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(unauthorizedResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    await waitFor(() => {
      expect(window.location.pathname).toBe("/login");
    });

    expect(authService.getSnapshot()).toBe(false);

    expect(
      screen.getByRole("heading", {
        name: "Sign in",
      }),
    ).toBeInTheDocument();
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
        name: "Accounts",
        exact: true,
      }),
    ).toBeInTheDocument();

    await screen.findByRole("button", { name: "Logout" });

    expect(window.location.pathname).toBe("/admin");

    screen
      .getByRole("link", {
        name: "Accounts",
        exact: true,
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

    expect(screen.getByRole("button", { name: "Logout" })).toBeInTheDocument();
    expect(
      screen.getByRole("heading", {
        name: "Administrator Accounts",
      }),
    ).toBeInTheDocument();
  });
});
