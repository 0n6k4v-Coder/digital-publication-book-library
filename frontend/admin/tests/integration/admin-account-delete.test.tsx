import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accountId = "01900000-0000-7000-8000-000000000001";

const accessToken = "opaque-access-token";

function problemResponse(status: number, code: string): Response {
  return new Response(
    JSON.stringify({
      type: "https://example.invalid/problems/account",
      title: "Account error",
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

function accountResponse(deletedAt: string | null = null): Response {
  return new Response(
    JSON.stringify({
      id: accountId,
      email: "admin@example.com",
      display_name: "Library Administrator",
      status: deletedAt === null ? "active" : "inactive",
      created_at: "2026-09-23T10:00:00Z",
      updated_at: "2026-09-23T10:00:00Z",
      deleted_at: deletedAt,
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

function listResponse(): Response {
  return new Response(
    JSON.stringify({
      items: [],
      page: 1,
      page_size: 20,
      total: 0,
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
  window.history.replaceState(
    {},
    "",
    `/admin/accounts/${accountId}/delete?return_to=%2Fadmin%2Faccounts%3Fpage%3D2`,
  );

  authService.clearClientState();

  vi.stubGlobal("fetch", vi.fn());

  vi.mocked(fetch).mockResolvedValueOnce(problemResponse(401, "UNAUTHORIZED"));

  await authService.retryBootstrap();
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

async function signIn(): Promise<void> {
  vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

  await authService.login({
    email: "admin@example.com",
    password: "example-secure-password",
  });
}

describe("Admin Account Delete integration", () => {
  it("loads inside the Admin Shell and performs a soft DELETE with 204 success", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse())
      .mockResolvedValueOnce(new Response(null, { status: 204 }))
      .mockResolvedValueOnce(listResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    expect(
      await screen.findByRole("heading", {
        name: "Delete Administrator Account",
      }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("complementary", {
        name: "Admin application",
      }),
    ).toBeInTheDocument();

    screen
      .getByRole("button", {
        name: "Soft-delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Delete Account",
      })
      .click();

    await waitFor(() => {
      expect(window.location.pathname).toBe("/admin/accounts");
    });

    const deleteCall = vi
      .mocked(fetch)
      .mock.calls.find(
        ([url, init]) =>
          url === `/admin/accounts/${accountId}` && init?.method === "DELETE",
      );

    expect(deleteCall).toBeDefined();

    expect(new Headers(deleteCall?.[1]?.headers).get("authorization")).toBe(
      `Bearer ${accessToken}`,
    );

    expect(deleteCall?.[1]?.body).toBeUndefined();

    expect(String(deleteCall?.[0])).not.toContain(accessToken);
  });

  it("can hard-delete a target that is unavailable to the normal GET lookup", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(problemResponse(404, "ACCOUNT_NOT_FOUND"))
      .mockResolvedValueOnce(new Response(null, { status: 204 }))
      .mockResolvedValueOnce(listResponse());

    await signIn();
    render(<App />);

    expect(
      await screen.findByRole("heading", {
        name: "Account unavailable",
      }),
    ).toBeInTheDocument();

    screen
      .getByRole("button", {
        name: "Permanently delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Permanently Delete",
      })
      .click();

    await waitFor(() => {
      expect(window.location.pathname).toBe("/admin/accounts");
    });

    const purgeCall = vi
      .mocked(fetch)
      .mock.calls.find(
        ([url, init]) =>
          url === `/admin/accounts/${accountId}/purge` &&
          init?.method === "DELETE",
      );

    expect(purgeCall).toBeDefined();

    expect(purgeCall?.[1]?.body).toBeUndefined();
  });

  it("keeps authentication state on a 403 and does not logout", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse())
      .mockResolvedValueOnce(problemResponse(403, "ACCOUNT_DELETE_FORBIDDEN"));

    await signIn();
    render(<App />);

    screen
      .getByRole("button", {
        name: "Soft-delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Delete Account",
      })
      .click();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "You do not have permission to soft-delete this administrator account.",
    );

    expect(window.location.pathname).toBe(
      `/admin/accounts/${accountId}/delete`,
    );

    expect(authService.getSnapshot().authStatus).toBe("authenticated");
  });

  it("renders ACCOUNT_ALREADY_DELETED distinctly", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse())
      .mockResolvedValueOnce(problemResponse(409, "ACCOUNT_ALREADY_DELETED"));

    await signIn();
    render(<App />);

    screen
      .getByRole("button", {
        name: "Soft-delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Delete Account",
      })
      .click();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "has already been soft-deleted",
    );
  });

  it("renders LAST_ACTIVE_ADMINISTRATOR distinctly for hard deletion", async () => {
    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse())
      .mockResolvedValueOnce(problemResponse(409, "LAST_ACTIVE_ADMINISTRATOR"));

    await signIn();
    render(<App />);

    screen
      .getByRole("button", {
        name: "Permanently delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Permanently Delete",
      })
      .click();

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The last active administrator cannot be deleted.",
    );
  });

  it("rejects an external return_to and never navigates off-origin", async () => {
    window.history.replaceState(
      {},
      "",
      `/admin/accounts/${accountId}/delete?return_to=${encodeURIComponent(
        "https://evil.example/admin/accounts",
      )}`,
    );

    vi.mocked(fetch)
      .mockResolvedValueOnce(loginResponse())
      .mockResolvedValueOnce(accountResponse())
      .mockResolvedValueOnce(new Response(null, { status: 204 }))
      .mockResolvedValueOnce(listResponse());

    await signIn();
    render(<App />);

    screen
      .getByRole("button", {
        name: "Soft-delete account",
      })
      .click();

    screen
      .getByRole("button", {
        name: "Delete Account",
      })
      .click();

    await waitFor(() => {
      expect(window.location.pathname).toBe("/admin/accounts");
    });

    expect(window.location.origin).toBe("http://localhost:5173");
  });
});
