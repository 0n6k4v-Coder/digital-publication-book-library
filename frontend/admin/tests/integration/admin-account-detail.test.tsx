import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";
const refreshToken = "opaque-refresh-token";

const account = {
  id: "01900000-0000-7000-8000-000000000001",
  email: "admin@example.com",
  display_name: "Library Administrator",
  status: "active",
  created_at: "2026-09-23T10:00:00Z",
  updated_at: "2026-09-23T10:00:00Z",
  deleted_at: null,
};

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

function accountResponse(): Response {
  return new Response(JSON.stringify(account), {
    status: 200,
    headers: {
      "Cache-Control": "no-store",
      "Content-Type": "application/json",
    },
  });
}

beforeEach(() => {
  window.history.replaceState({}, "", `/admin/accounts/${account.id}/edit`);
  authService.clearClientState();
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("Account Detail integration", () => {
  it("redirects an unauthenticated detail request to login", async () => {
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

  it("renders inside AdminShell and loads the authoritative Account", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", {
        name: "Edit Administrator Account",
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
      screen.getByRole("button", {
        name: "Logout",
      }),
    ).toBeInTheDocument();

    const accountCall = vi.mocked(fetch).mock.calls[1];
    const headers = new Headers(accountCall[1]?.headers);

    expect(accountCall[0]).toBe(`/admin/accounts/${account.id}`);
    expect(accountCall[1]?.method).toBe("GET");
    expect(accountCall[1]?.cache).toBe("no-store");
    expect(headers.get("authorization")).toBe(`Bearer ${accessToken}`);

    expect(String(accountCall[0])).not.toContain(accessToken);
    expect(String(accountCall[0])).not.toContain(refreshToken);
    expect(document.body).not.toHaveTextContent(accessToken);
    expect(document.body).not.toHaveTextContent(refreshToken);
  });

  it("uses the exact default Back destination when no return_to is supplied", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("link", {
        name: "Back to Administrator Accounts",
      }),
    ).toHaveAttribute("href", "/admin/accounts");
  });
});
