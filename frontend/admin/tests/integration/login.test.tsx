import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";
const refreshToken = "opaque-refresh-token";

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

beforeEach(() => {
  window.history.replaceState({}, "", "/login");
  authService.clearClientState();
  vi.stubGlobal("fetch", vi.fn());
});

afterEach(() => {
  authService.clearClientState();
  vi.unstubAllGlobals();
});

describe("login integration", () => {
  it("sends the exact login request and establishes client authentication state on success", async () => {
    const user = userEvent.setup();
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockResolvedValueOnce(loginResponse());

    render(<App />);

    await user.type(screen.getByLabelText("Email"), "admin@example.com");
    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );
    await user.click(screen.getByRole("button", { name: "Sign In" }));

    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("/auth/login");
    expect(init).toMatchObject({
      method: "POST",
      cache: "no-store",
    });
    expect(new Headers(init?.headers).get("content-type")).toBe(
      "application/json",
    );
    expect(init?.body).toBe(
      JSON.stringify({
        email: "admin@example.com",
        password: "example-secure-password",
      }),
    );
    expect(document.body).not.toHaveTextContent(accessToken);
    expect(document.body).not.toHaveTextContent(refreshToken);
  });

  it.each([
    [400, "INVALID_REQUEST", "Please check the required fields and try again."],
    [401, "INVALID_CREDENTIALS", "The email or password is incorrect."],
    [
      429,
      "AUTHENTICATION_RATE_LIMITED",
      "Too many sign-in attempts. Please try again later.",
    ],
  ] as const)(
    "keeps the user on login for %s %s",
    async (status, code, expectedMessage) => {
      const user = userEvent.setup();
      vi.mocked(fetch).mockResolvedValueOnce(problemResponse(status, code));

      render(<App />);

      await user.type(screen.getByLabelText("Email"), "admin@example.com");
      await user.type(
        screen.getByLabelText("Password"),
        "example-secure-password",
      );
      await user.click(screen.getByRole("button", { name: "Sign In" }));

      expect(await screen.findByRole("alert")).toHaveTextContent(
        expectedMessage,
      );
      expect(window.location.pathname).toBe("/login");
      expect(
        screen.queryByText("Digital Publication & Book Library"),
      ).not.toBeInTheDocument();
      expect(
        screen.queryByText("server detail must not reach the UI"),
      ).not.toBeInTheDocument();
    },
  );

  it("prevents duplicate login requests while authentication is pending", async () => {
    const user = userEvent.setup();
    let resolveLogin!: (response: Response) => void;

    vi.mocked(fetch).mockImplementationOnce(
      () =>
        new Promise<Response>((resolve) => {
          resolveLogin = resolve;
        }),
    );

    render(<App />);

    await user.type(screen.getByLabelText("Email"), "admin@example.com");
    await user.type(
      screen.getByLabelText("Password"),
      "example-secure-password",
    );
    await user.click(screen.getByRole("button", { name: "Sign In" }));
    await user.click(screen.getByRole("button", { name: "Signing in…" }));

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(1);

    resolveLogin(loginResponse());
    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );
  });

  it("redirects unauthenticated access to /admin back to /login", async () => {
    window.history.replaceState({}, "", "/admin");

    render(<App />);

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );
    expect(window.location.pathname).toBe("/login");
  });

  it("redirects authenticated access to /login to /admin", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());
    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });
    window.history.replaceState({}, "", "/login");

    render(<App />);

    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );
    expect(window.location.pathname).toBe("/admin");
  });

  it("logs out through the authenticated endpoint, including bearer authorization", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    render(<App />);
    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );

    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(null, {
        status: 204,
        headers: {
          "Cache-Control": "no-store",
        },
      }),
    );

    await user.click(screen.getByRole("button", { name: "Logout" }));

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );
    expect(window.location.pathname).toBe("/login");

    const [, logoutInit] = vi.mocked(fetch).mock.calls[1];
    expect(logoutInit?.method).toBe("POST");
    expect(logoutInit?.headers).toEqual(
      expect.objectContaining({ Authorization: `Bearer ${accessToken}` }),
    );
    expect(document.body).not.toHaveTextContent(accessToken);
    expect(document.body).not.toHaveTextContent(refreshToken);
  });

  it("clears client authentication and returns to login when logout receives 401", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    render(<App />);
    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });
    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );

    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(401, "UNAUTHORIZED"),
    );
    await user.click(screen.getByRole("button", { name: "Logout" }));

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );
    expect(window.location.pathname).toBe("/login");
  });

  it("keeps the authenticated shell available after a non-401 logout failure", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    render(<App />);
    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });
    await waitFor(() =>
      expect(
        screen.getByText("Digital Publication & Book Library"),
      ).toBeInTheDocument(),
    );

    vi.mocked(fetch).mockResolvedValueOnce(problemResponse(500, "UNKNOWN"));
    await user.click(screen.getByRole("button", { name: "Logout" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We could not sign you out. Please try again.",
    );
    expect(window.location.pathname).toBe("/admin");
    expect(screen.getByRole("button", { name: "Logout" })).toBeEnabled();
  });
});
