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
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${account.id}/edit?return_to=%2Fadmin%2Faccounts%3Fpage%3D2`,
  );
  authService.clearClientState();
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("Account Edit integration", () => {
  it("redirects an unauthenticated edit request to login", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          status: 401,
          code: "INVALID_REFRESH_TOKEN",
        }),
        {
          status: 401,
          headers: {
            "Cache-Control": "no-store",
            "Content-Type": "application/problem+json",
          },
        },
      ),
    );

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
    expect(vi.mocked(fetch).mock.calls[0][0]).toBe("/auth/refresh");
  });

  it("renders Account Edit inside the Admin Shell with all three mutation sections", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            status: 401,
            code: "INVALID_REFRESH_TOKEN",
          }),
          {
            status: 401,
            headers: {
              "Cache-Control": "no-store",
              "Content-Type": "application/problem+json",
            },
          },
        ),
      )
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse());

    await authService.retryBootstrap();

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
      }),
    ).toHaveAttribute("aria-current", "page");

    expect(
      screen.getByRole("heading", {
        name: "Administrator account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("heading", {
        name: "Administrator email address",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("heading", {
        name: "Administrator password",
      }),
    ).toBeInTheDocument();

    expect(
      screen.queryByRole("button", {
        name: /deactivate|activate|restore|delete/i,
      }),
    ).not.toBeInTheDocument();

    const backLink = screen.getByRole("link", {
      name: "Back to Administrator Accounts",
    });

    expect(backLink).toHaveAttribute("href", "/admin/accounts?page=2");

    const accountCall = vi.mocked(fetch).mock.calls[2];
    const headers = new Headers(accountCall[1]?.headers);

    expect(accountCall[0]).toBe(`/admin/accounts/${account.id}`);
    expect(accountCall[1]?.method).toBe("GET");
    expect(accountCall[1]?.cache).toBe("no-store");
    expect(headers.get("authorization")).toBe(`Bearer ${accessToken}`);

    expect(String(accountCall[0])).not.toContain(accessToken);
    expect(String(accountCall[0])).not.toContain(refreshToken);
  });
});
