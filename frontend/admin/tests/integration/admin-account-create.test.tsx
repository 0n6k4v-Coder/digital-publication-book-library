import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";
const accountId = "01900000-0000-7000-8000-000000000010";

function problemResponse(status: number, code: string): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/account-create",
      title: "Account create failure",
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

function createdAccountResponse(): Response {
  return new Response(
    JSON.stringify({
      id: accountId,
      email: "created@example.com",
      display_name: null,
      status: "active",
      created_at: "2026-09-27T02:00:00Z",
      updated_at: "2026-09-27T02:00:00Z",
      deleted_at: null,
    }),
    {
      status: 201,
      headers: {
        "Cache-Control": "no-store",
        "Content-Type": "application/json",
        Location: `/admin/accounts/${accountId}`,
      },
    },
  );
}

function accountDetailResponse(): Response {
  return new Response(
    JSON.stringify({
      id: accountId,
      email: "created@example.com",
      display_name: null,
      status: "active",
      created_at: "2026-09-27T02:00:00Z",
      updated_at: "2026-09-27T02:00:00Z",
      deleted_at: null,
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
  vi.mocked(fetch).mockResolvedValueOnce(problemResponse(401, "UNAUTHORIZED"));

  await authService.retryBootstrap();
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("Account Create integration", () => {
  it("renders inside the Admin Shell and creates an Account through the documented API", async () => {
    const user = userEvent.setup();

    window.history.replaceState({}, "", "/admin/accounts/create");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(createdAccountResponse())
      .mockResolvedValueOnce(accountDetailResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", {
        level: 1,
        name: "Create Administrator Account",
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

    await user.type(screen.getByLabelText("Email"), "created@example.com");

    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );

    await user.click(
      screen.getByRole("button", {
        name: "Create Account",
      }),
    );

    await waitFor(() => {
      expect(window.location.pathname).toBe(`/admin/accounts/${accountId}`);
    });

    expect(
      await screen.findByRole("heading", {
        name: "Administrator Account",
      }),
    ).toBeInTheDocument();

    const createCall = vi.mocked(fetch).mock.calls[2];

    expect(createCall[0]).toBe("/admin/accounts");
    expect(createCall[1]?.method).toBe("POST");
    expect(new Headers(createCall[1]?.headers).get("authorization")).toBe(
      `Bearer ${accessToken}`,
    );
    expect(createCall[1]?.body).toBe(
      JSON.stringify({
        email: "created@example.com",
        password: "example-secure-password",
      }),
    );

    expect(String(createCall[0])).not.toContain(accessToken);
    expect(String(createCall[0])).not.toContain("example-secure-password");
    expect(String(createCall[1]?.body)).not.toContain("display_name");
    expect(document.body).not.toHaveTextContent(accessToken);
  });

  it("keeps the authenticated shell mounted after a 403", async () => {
    const user = userEvent.setup();

    window.history.replaceState({}, "", "/admin/accounts/create");

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(problemResponse(403, "ACCOUNT_CREATE_FORBIDDEN"));

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    await screen.findByRole("heading", {
      name: "Create Administrator Account",
    });

    await user.type(screen.getByLabelText("Email"), "created@example.com");

    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );

    await user.click(
      screen.getByRole("button", {
        name: "Create Account",
      }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to create administrator accounts.",
    );

    expect(window.location.pathname).toBe("/admin/accounts/create");
    expect(authService.getSnapshot().authStatus).toBe("authenticated");

    expect(
      screen.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeInTheDocument();

    expect(document.body).not.toHaveTextContent(
      "server detail must not reach the UI",
    );
  });
});
