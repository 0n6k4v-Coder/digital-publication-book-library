import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import { authService } from "../../src/services/auth";

const accessToken = "opaque-access-token";

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

function logoutResponse(): Response {
  return new Response(null, {
    status: 204,
    headers: {
      "Cache-Control": "no-store",
    },
  });
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

describe("Admin Shell integration", () => {
  it("renders the shell for an authenticated user and keeps main content empty", async () => {
    window.history.replaceState({}, "", "/admin");
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    render(<App />);

    await waitFor(() =>
      expect(
        screen.getByRole("heading", {
          name: "Digital Publication & Book Library",
        }),
      ).toBeInTheDocument(),
    );

    expect(
      screen.getByRole("complementary", { name: "Admin application" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Logout" })).toBeInTheDocument();
    expect(
      screen.getByRole("main", { name: "Main content" }),
    ).toBeEmptyDOMElement();
    expect(window.location.pathname).toBe("/admin");
  });

  it("redirects unauthenticated access to /admin to /login", async () => {
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
        screen.getByRole("heading", {
          name: "Digital Publication & Book Library",
        }),
      ).toBeInTheDocument(),
    );

    expect(window.location.pathname).toBe("/admin");
  });

  it("sends POST /auth/logout with the browser-managed authentication credential and redirects after 204", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    window.history.replaceState({}, "", "/admin");
    render(<App />);

    expect(await screen.findByRole("button", { name: "Logout" })).toBeEnabled();

    vi.mocked(fetch).mockResolvedValueOnce(logoutResponse());

    await user.click(screen.getByRole("button", { name: "Logout" }));

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );

    expect(window.location.pathname).toBe("/login");
    expect(authService.getSnapshot().authStatus).toBe("unauthenticated");

    const [url, init] = vi.mocked(fetch).mock.calls[2];

    expect(url).toBe("/auth/logout");
    expect(init?.method).toBe("POST");
    expect(init?.credentials).toBe("include");
    expect(new Headers(init?.headers).get("authorization")).toBeNull();
    expect(init?.body).toBeUndefined();
    expect(url).not.toContain(accessToken);
    expect(document.body).not.toHaveTextContent(accessToken);
  });

  it("clears authentication state and redirects after logout 401", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    window.history.replaceState({}, "", "/admin");
    render(<App />);

    await screen.findByRole("button", { name: "Logout" });

    vi.mocked(fetch).mockResolvedValueOnce(
      problemResponse(401, "UNAUTHORIZED"),
    );

    await user.click(screen.getByRole("button", { name: "Logout" }));

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );

    expect(authService.getSnapshot().authStatus).toBe("unauthenticated");
    expect(window.location.pathname).toBe("/login");
  });

  it("keeps the authenticated shell after a non-401 logout failure", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    window.history.replaceState({}, "", "/admin");
    render(<App />);

    await screen.findByRole("button", { name: "Logout" });

    vi.mocked(fetch).mockResolvedValueOnce(problemResponse(500, "UNKNOWN"));

    await user.click(screen.getByRole("button", { name: "Logout" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "We could not sign you out. Please try again.",
    );
    expect(window.location.pathname).toBe("/admin");
    expect(authService.getSnapshot().authStatus).toBe("authenticated");
    expect(screen.getByRole("button", { name: "Logout" })).toBeEnabled();
  });

  it("prevents duplicate logout requests while the first request is pending", async () => {
    const user = userEvent.setup();
    vi.mocked(fetch).mockResolvedValueOnce(loginResponse());

    await authService.login({
      email: "admin@example.com",
      password: "example-secure-password",
    });

    window.history.replaceState({}, "", "/admin");
    render(<App />);

    await screen.findByRole("button", { name: "Logout" });

    let resolveLogout!: (response: Response) => void;

    vi.mocked(fetch).mockImplementationOnce(
      () =>
        new Promise<Response>((resolve) => {
          resolveLogout = resolve;
        }),
    );

    await user.click(screen.getByRole("button", { name: "Logout" }));

    expect(screen.getByRole("button", { name: "Signing out…" })).toBeDisabled();

    const secondLogout = authService.logout();

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(3);

    resolveLogout(logoutResponse());
    await secondLogout;

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Sign in" }),
      ).toBeInTheDocument(),
    );

    expect(vi.mocked(fetch)).toHaveBeenCalledTimes(3);
  });
});
